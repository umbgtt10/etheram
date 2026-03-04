// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::common_types::node_role::NodeRole;
use raft_node::context::context_dto::RaftContext;
use std::collections::BTreeMap;

pub fn make_ctx(peer_id: u64, peers: Vec<u64>, role: NodeRole) -> RaftContext<Vec<u8>> {
    RaftContext {
        peer_id,
        current_term: 1,
        voted_for: None,
        log: Vec::new(),
        commit_index: 0,
        last_applied: 0,
        role,
        leader_id: None,
        peers,
        match_index: BTreeMap::new(),
        next_index: BTreeMap::new(),
        snapshot: None,
    }
}

pub fn make_ctx_with_term(
    peer_id: u64,
    peers: Vec<u64>,
    role: NodeRole,
    term: u64,
) -> RaftContext<Vec<u8>> {
    let mut ctx = make_ctx(peer_id, peers, role);
    ctx.current_term = term;
    ctx
}
