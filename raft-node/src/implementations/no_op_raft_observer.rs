// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message_source::MessageSource;
use crate::common_types::node_role::NodeRole;
use crate::observer::EventLevel;
use crate::observer::RaftActionKind;
use crate::observer::RaftObserver;
use etheram_core::types::PeerId;

pub struct NoOpRaftObserver;

impl NoOpRaftObserver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpRaftObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl RaftObserver for NoOpRaftObserver {
    fn min_level(&self) -> EventLevel {
        EventLevel::None
    }

    fn node_started(&mut self, _peer_id: PeerId) {}

    fn message_received(&mut self, _peer_id: PeerId, _source: &MessageSource) {}

    fn context_built(&mut self, _peer_id: PeerId, _term: u64, _role: NodeRole, _log_length: usize) {
    }

    fn action_emitted(&mut self, _peer_id: PeerId, _kind: &RaftActionKind) {}

    fn mutation_applied(&mut self, _peer_id: PeerId, _kind: &RaftActionKind) {}

    fn output_executed(&mut self, _peer_id: PeerId, _kind: &RaftActionKind) {}

    fn step_completed(&mut self, _peer_id: PeerId, _processed: bool) {}
}
