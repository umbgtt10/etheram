// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::brain::protocol::unified_message::Message;
use raft_node::context::context_builder::RaftContextBuilder;
use raft_node::context::context_dto::RaftContext;
use raft_node::state::raft_state::RaftState;

pub struct EagerRaftContextBuilder {
    peer_id: PeerId,
    peers: Vec<PeerId>,
}

impl EagerRaftContextBuilder {
    pub fn new(peer_id: PeerId, peers: Vec<PeerId>) -> Self {
        Self { peer_id, peers }
    }
}

impl<P: Clone + 'static> RaftContextBuilder<P> for EagerRaftContextBuilder {
    fn build(
        &self,
        state: &RaftState<P>,
        _peer_id: PeerId,
        _source: &MessageSource,
        _message: &Message<P>,
    ) -> RaftContext<P> {
        let current_term = state.query_current_term();
        let voted_for = state.query_voted_for();
        let log = state.query_all_entries();
        let commit_index = state.query_commit_index();
        let last_applied = state.query_last_applied();
        let role = state.query_role();
        let leader_id = state.query_leader_id();
        let snapshot = state.query_snapshot();
        let match_index = state.query_all_match_index();
        let next_index = state.query_all_next_index();
        RaftContext {
            peer_id: self.peer_id,
            current_term,
            voted_for,
            log,
            commit_index,
            last_applied,
            role,
            leader_id,
            peers: self.peers.clone(),
            match_index,
            next_index,
            snapshot,
        }
    }
}
