// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::message::RaftMessage;
use crate::common_types::node_role::NodeRole;
use crate::context::context_dto::RaftContext;
use crate::implementations::raft::common;
use crate::implementations::raft::common::ELECTION_TIMEOUT_MS;
use crate::implementations::raft::common::HEARTBEAT_INTERVAL_MS;
use crate::implementations::raft::raft_protocol::RaftProtocol;
use crate::implementations::raft::replication;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::vec;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

pub fn handle_election_timeout<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
) -> Vec<RaftAction<P>> {
    if ctx.role == NodeRole::Leader {
        return vec![RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::ElectionTimeout,
            delay: ELECTION_TIMEOUT_MS,
        }];
    }
    if ctx.peers.is_empty() {
        start_election(protocol, ctx)
    } else {
        start_pre_vote(protocol, ctx)
    }
}

fn start_pre_vote<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
) -> Vec<RaftAction<P>> {
    protocol.pre_votes_received.clear();
    protocol.pre_votes_received.insert(ctx.peer_id);
    vec![
        RaftAction::TransitionRole(NodeRole::PreCandidate),
        RaftAction::BroadcastMessage {
            message: RaftMessage::PreVoteRequest {
                next_term: ctx.current_term + 1,
                candidate_id: ctx.peer_id,
                last_log_index: common::last_log_index(ctx),
                last_log_term: common::last_log_term(ctx),
            },
        },
        RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::ElectionTimeout,
            delay: ELECTION_TIMEOUT_MS,
        },
    ]
}

pub fn start_election<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
) -> Vec<RaftAction<P>> {
    let new_term = ctx.current_term + 1;
    protocol.votes_received.clear();
    protocol.votes_received.insert(ctx.peer_id);
    protocol.pre_votes_received.clear();
    let mut actions = vec![
        RaftAction::SetTerm(new_term),
        RaftAction::SetVotedFor(Some(ctx.peer_id)),
        RaftAction::TransitionRole(NodeRole::Candidate),
        RaftAction::BroadcastMessage {
            message: RaftMessage::RequestVote {
                term: new_term,
                candidate_id: ctx.peer_id,
                last_log_index: common::last_log_index(ctx),
                last_log_term: common::last_log_term(ctx),
            },
        },
        RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::ElectionTimeout,
            delay: ELECTION_TIMEOUT_MS,
        },
    ];
    if ctx.peers.is_empty() {
        actions.extend(become_leader(protocol, ctx));
    }
    actions
}

pub fn become_leader<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
) -> Vec<RaftAction<P>> {
    protocol.votes_received.clear();
    let last_idx = common::last_log_index(ctx);
    let mut actions = vec![
        RaftAction::TransitionRole(NodeRole::Leader),
        RaftAction::SetLeaderId(Some(ctx.peer_id)),
        RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::Heartbeat,
            delay: HEARTBEAT_INTERVAL_MS,
        },
    ];
    for &peer in &ctx.peers {
        actions.push(RaftAction::UpdateMatchIndex {
            peer_id: peer,
            index: 0,
        });
        actions.push(RaftAction::UpdateNextIndex {
            peer_id: peer,
            index: last_idx + 1,
        });
    }
    actions.extend(replication::send_all_peers_with_next(ctx, last_idx + 1));
    actions
}

pub fn handle_pre_vote_request<P: Clone>(
    ctx: &RaftContext<P>,
    from: PeerId,
    next_term: u64,
    _candidate_id: PeerId,
    last_log_index: u64,
    last_log_term: u64,
) -> Vec<RaftAction<P>> {
    let our_last_term = common::last_log_term(ctx);
    let our_last_index = common::last_log_index(ctx);
    let vote_granted = next_term > ctx.current_term
        && common::log_is_up_to_date(our_last_term, our_last_index, last_log_term, last_log_index);
    vec![RaftAction::SendMessage {
        to: from,
        message: RaftMessage::PreVoteResponse {
            term: ctx.current_term,
            vote_granted,
        },
    }]
}

pub fn handle_pre_vote_response<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    from: PeerId,
    vote_granted: bool,
) -> Vec<RaftAction<P>> {
    if ctx.role != NodeRole::PreCandidate {
        return Vec::new();
    }
    if !vote_granted {
        return Vec::new();
    }
    protocol.pre_votes_received.insert(from);
    let quorum = common::quorum_size(ctx.peers.len());
    if protocol.pre_votes_received.len() >= quorum {
        start_election(protocol, ctx)
    } else {
        Vec::new()
    }
}

pub fn handle_request_vote<P: Clone>(
    ctx: &RaftContext<P>,
    from: PeerId,
    term: u64,
    candidate_id: PeerId,
    last_log_index: u64,
    last_log_term: u64,
) -> Vec<RaftAction<P>> {
    let our_last_term = common::last_log_term(ctx);
    let our_last_index = common::last_log_index(ctx);
    let mut actions = Vec::new();

    if term > ctx.current_term {
        actions.extend(common::step_down::<P>(term));
    }

    let effective_term = term.max(ctx.current_term);
    if term < ctx.current_term {
        actions.push(RaftAction::SendMessage {
            to: from,
            message: RaftMessage::RequestVoteResponse {
                term: ctx.current_term,
                vote_granted: false,
            },
        });
        return actions;
    }

    let already_voted_other = ctx.voted_for.is_some() && ctx.voted_for != Some(candidate_id);
    let log_ok =
        common::log_is_up_to_date(our_last_term, our_last_index, last_log_term, last_log_index);
    let grant = !already_voted_other && log_ok;

    if grant {
        actions.push(RaftAction::SetVotedFor(Some(candidate_id)));
        actions.push(RaftAction::ScheduleTimeout {
            event: RaftTimerEvent::ElectionTimeout,
            delay: ELECTION_TIMEOUT_MS,
        });
    }

    actions.push(RaftAction::SendMessage {
        to: from,
        message: RaftMessage::RequestVoteResponse {
            term: effective_term,
            vote_granted: grant,
        },
    });
    actions
}

pub fn handle_request_vote_response<P: Clone>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    from: PeerId,
    term: u64,
    vote_granted: bool,
) -> Vec<RaftAction<P>> {
    if term > ctx.current_term {
        return common::step_down(term);
    }
    if ctx.role != NodeRole::Candidate {
        return Vec::new();
    }
    if term != ctx.current_term {
        return Vec::new();
    }
    if !vote_granted {
        return Vec::new();
    }
    protocol.votes_received.insert(from);
    let quorum = common::quorum_size(ctx.peers.len());
    if protocol.votes_received.len() >= quorum {
        become_leader(protocol, ctx)
    } else {
        Vec::new()
    }
}
