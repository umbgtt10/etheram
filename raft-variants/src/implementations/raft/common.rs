// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use etheram_core::types::{ClientId, PeerId};
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::log_entry::LogEntry;
use raft_node::common_types::node_role::NodeRole;
use raft_node::context::context_dto::RaftContext;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;

pub const ELECTION_TIMEOUT_MS: u64 = 300;
pub const HEARTBEAT_INTERVAL_MS: u64 = 100;

pub fn last_log_index<P>(ctx: &RaftContext<P>) -> u64 {
    if let Some(e) = ctx.log.last() {
        e.index
    } else if let Some(ref snap) = ctx.snapshot {
        snap.last_included_index
    } else {
        0
    }
}

pub fn last_log_term<P>(ctx: &RaftContext<P>) -> u64 {
    if let Some(e) = ctx.log.last() {
        e.term
    } else if let Some(ref snap) = ctx.snapshot {
        snap.last_included_term
    } else {
        0
    }
}

pub fn quorum_size(peer_count: usize) -> usize {
    peer_count.div_ceil(2) + 1
}

pub fn log_is_up_to_date(
    our_last_term: u64,
    our_last_index: u64,
    their_last_term: u64,
    their_last_index: u64,
) -> bool {
    their_last_term > our_last_term
        || (their_last_term == our_last_term && their_last_index >= our_last_index)
}

pub fn step_down<P: Clone>(new_term: u64) -> Vec<RaftAction<P>> {
    vec![
        RaftAction::SetTerm(new_term),
        RaftAction::SetVotedFor(None),
        RaftAction::TransitionRole(NodeRole::Follower),
        RaftAction::SetLeaderId(None),
    ]
}

pub fn prev_log_term_at<P>(ctx: &RaftContext<P>, index: u64) -> u64 {
    if index == 0 {
        return 0;
    }
    if let Some(ref snap) = ctx.snapshot {
        if index == snap.last_included_index {
            return snap.last_included_term;
        }
    }
    ctx.log
        .iter()
        .find(|e| e.index == index)
        .map(|e| e.term)
        .unwrap_or(0)
}

pub fn build_append_entries_for_peer<P: Clone>(
    ctx: &RaftContext<P>,
    to: PeerId,
    next_idx: u64,
) -> RaftAction<P> {
    let prev_log_index = next_idx.saturating_sub(1);
    let prev_log_term = prev_log_term_at(ctx, prev_log_index);
    let entries: Vec<LogEntry<P>> = ctx
        .log
        .iter()
        .filter(|e| e.index >= next_idx)
        .cloned()
        .collect();
    RaftAction::SendMessage {
        to,
        message: RaftMessage::AppendEntries {
            term: ctx.current_term,
            leader_id: ctx.peer_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit: ctx.commit_index,
        },
    }
}

pub fn advance_commit_index<P: Clone + AsRef<[u8]>>(
    ctx: &RaftContext<P>,
    updated_peer: PeerId,
    updated_match_index: u64,
    pending_client_entries: &BTreeMap<u64, ClientId>,
) -> Vec<RaftAction<P>> {
    let last_idx = last_log_index(ctx);
    let quorum = quorum_size(ctx.peers.len());
    let mut new_commit = ctx.commit_index;

    for idx in (ctx.commit_index + 1)..=last_idx {
        let entry_term = ctx.log.iter().find(|e| e.index == idx).map(|e| e.term);
        if entry_term != Some(ctx.current_term) {
            continue;
        }
        let count = ctx
            .peers
            .iter()
            .filter(|&&peer| {
                let match_idx = if peer == updated_peer {
                    updated_match_index
                } else {
                    ctx.match_index.get(&peer).copied().unwrap_or(0)
                };
                match_idx >= idx
            })
            .count();
        if count + 1 >= quorum {
            new_commit = idx;
        }
    }

    if new_commit <= ctx.commit_index {
        return Vec::new();
    }

    let mut actions = vec![RaftAction::AdvanceCommitIndex(new_commit)];
    for entry in ctx
        .log
        .iter()
        .filter(|e| e.index > ctx.commit_index && e.index <= new_commit)
    {
        let client_id = pending_client_entries.get(&entry.index).copied();
        actions.push(RaftAction::ApplyToStateMachine {
            client_id,
            index: entry.index,
            payload_bytes: entry.payload.as_ref().to_vec(),
        });
        if let Some(cid) = client_id {
            actions.push(RaftAction::SendClientResponse {
                client_id: cid,
                response: RaftClientResponse::Applied(Vec::new()),
            });
        }
    }
    actions
}

pub fn apply_committed_entries_follower<P: Clone + AsRef<[u8]>>(
    ctx: &RaftContext<P>,
    new_commit: u64,
    newly_appended: &[LogEntry<P>],
) -> Vec<RaftAction<P>> {
    if new_commit <= ctx.commit_index {
        return Vec::new();
    }
    let mut actions = vec![RaftAction::AdvanceCommitIndex(new_commit)];
    let combined: Vec<&LogEntry<P>> = ctx
        .log
        .iter()
        .chain(newly_appended.iter())
        .filter(|e| e.index > ctx.commit_index && e.index <= new_commit)
        .collect();
    let mut seen = alloc::collections::BTreeSet::new();
    for entry in combined {
        if seen.insert(entry.index) {
            actions.push(RaftAction::ApplyToStateMachine {
                client_id: None,
                index: entry.index,
                payload_bytes: entry.payload.as_ref().to_vec(),
            });
        }
    }
    actions
}
