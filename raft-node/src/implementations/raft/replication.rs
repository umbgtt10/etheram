// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::message::RaftMessage;
use crate::common_types::log_entry::LogEntry;
use crate::common_types::node_role::NodeRole;
use crate::context::context_dto::RaftContext;
use crate::implementations::raft::common;
use crate::implementations::raft::common::ELECTION_TIMEOUT_MS;
use crate::implementations::raft::common::HEARTBEAT_INTERVAL_MS;
use crate::implementations::raft::raft_protocol::RaftProtocol;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::vec;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

pub fn handle_heartbeat<P: Clone>(ctx: &RaftContext<P>) -> Vec<RaftAction<P>> {
    if ctx.role != NodeRole::Leader {
        return Vec::new();
    }
    let mut actions = send_all_peers(ctx);
    actions.push(RaftAction::ScheduleTimeout {
        event: RaftTimerEvent::Heartbeat,
        delay: HEARTBEAT_INTERVAL_MS,
    });
    actions
}

pub fn send_all_peers<P: Clone>(ctx: &RaftContext<P>) -> Vec<RaftAction<P>> {
    ctx.peers
        .iter()
        .map(|&peer| {
            let next_idx = ctx.next_index.get(&peer).copied().unwrap_or(1);
            common::build_append_entries_for_peer(ctx, peer, next_idx)
        })
        .collect()
}

pub fn send_all_peers_with_next<P: Clone>(
    ctx: &RaftContext<P>,
    next_idx: u64,
) -> Vec<RaftAction<P>> {
    ctx.peers
        .iter()
        .map(|&peer| common::build_append_entries_for_peer(ctx, peer, next_idx))
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn handle_append_entries<P: Clone + AsRef<[u8]>>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    from: PeerId,
    term: u64,
    _leader_id: PeerId,
    prev_log_index: u64,
    prev_log_term: u64,
    entries: Vec<LogEntry<P>>,
    leader_commit: u64,
) -> Vec<RaftAction<P>> {
    if term < ctx.current_term {
        return vec![RaftAction::SendMessage {
            to: from,
            message: RaftMessage::AppendEntriesResponse {
                term: ctx.current_term,
                success: false,
                match_index: common::last_log_index(ctx),
            },
        }];
    }

    let mut actions = Vec::new();

    if term > ctx.current_term {
        actions.extend(common::step_down::<P>(term));
    } else if ctx.role == NodeRole::Candidate {
        actions.push(RaftAction::TransitionRole(NodeRole::Follower));
    }

    actions.push(RaftAction::SetLeaderId(Some(from)));
    actions.push(RaftAction::ScheduleTimeout {
        event: RaftTimerEvent::ElectionTimeout,
        delay: ELECTION_TIMEOUT_MS,
    });
    protocol.pending_client_entries.retain(|_, _| true);

    let effective_term = term.max(ctx.current_term);

    if !log_consistency_ok(ctx, prev_log_index, prev_log_term) {
        actions.push(RaftAction::SendMessage {
            to: from,
            message: RaftMessage::AppendEntriesResponse {
                term: effective_term,
                success: false,
                match_index: common::last_log_index(ctx),
            },
        });
        return actions;
    }

    let (truncate_at, entries_to_append) = find_append_point(ctx, prev_log_index, &entries);
    if let Some(idx) = truncate_at {
        actions.push(RaftAction::TruncateLogFrom(idx));
    }

    let last_new_index = if entries_to_append.is_empty() {
        common::last_log_index(ctx)
    } else {
        entries_to_append.last().map(|e| e.index).unwrap_or(0)
    };

    if !entries_to_append.is_empty() {
        actions.push(RaftAction::AppendEntries(entries_to_append.clone()));
    }

    let new_commit = leader_commit.min(last_new_index);
    actions.extend(common::apply_committed_entries_follower(
        ctx,
        new_commit,
        &entries_to_append,
    ));

    let final_match_index = last_new_index.max(prev_log_index);
    actions.push(RaftAction::SendMessage {
        to: from,
        message: RaftMessage::AppendEntriesResponse {
            term: effective_term,
            success: true,
            match_index: final_match_index,
        },
    });
    actions
}

fn log_consistency_ok<P>(ctx: &RaftContext<P>, prev_log_index: u64, prev_log_term: u64) -> bool {
    if prev_log_index == 0 {
        return true;
    }
    if let Some(ref snap) = ctx.snapshot {
        if prev_log_index == snap.last_included_index {
            return prev_log_term == snap.last_included_term;
        }
    }
    if let Some(entry) = ctx.log.iter().find(|e| e.index == prev_log_index) {
        entry.term == prev_log_term
    } else {
        false
    }
}

fn find_append_point<P: Clone>(
    ctx: &RaftContext<P>,
    prev_log_index: u64,
    entries: &[LogEntry<P>],
) -> (Option<u64>, Vec<LogEntry<P>>) {
    let mut truncate_at: Option<u64> = None;
    for new_entry in entries {
        if let Some(existing) = ctx.log.iter().find(|e| e.index == new_entry.index) {
            if existing.term != new_entry.term {
                truncate_at = Some(new_entry.index);
                break;
            }
        }
    }
    let start_from = if let Some(trunc) = truncate_at {
        trunc
    } else {
        prev_log_index + 1
    };
    let to_append: Vec<LogEntry<P>> = entries
        .iter()
        .filter(|e| e.index >= start_from)
        .cloned()
        .collect();
    (truncate_at, to_append)
}

pub fn handle_append_entries_response<P: Clone + AsRef<[u8]>>(
    protocol: &mut RaftProtocol<P>,
    ctx: &RaftContext<P>,
    from: PeerId,
    term: u64,
    success: bool,
    match_index: u64,
) -> Vec<RaftAction<P>> {
    if term > ctx.current_term {
        return common::step_down(term);
    }
    if ctx.role != NodeRole::Leader {
        return Vec::new();
    }
    if term != ctx.current_term {
        return Vec::new();
    }

    if success {
        let mut actions = vec![
            RaftAction::UpdateMatchIndex {
                peer_id: from,
                index: match_index,
            },
            RaftAction::UpdateNextIndex {
                peer_id: from,
                index: match_index + 1,
            },
        ];
        actions.extend(common::advance_commit_index(
            ctx,
            from,
            match_index,
            &protocol.pending_client_entries,
        ));
        let has_more = common::last_log_index(ctx) > match_index;
        if has_more {
            actions.push(common::build_append_entries_for_peer(
                ctx,
                from,
                match_index + 1,
            ));
        }
        actions
    } else {
        let current_next = ctx.next_index.get(&from).copied().unwrap_or(1);
        let decremented = current_next.saturating_sub(1).max(1);
        vec![
            RaftAction::UpdateNextIndex {
                peer_id: from,
                index: decremented,
            },
            common::build_append_entries_for_peer(ctx, from, decremented),
        ]
    }
}
