// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message_source::MessageSource;
use crate::common_types::types::{Hash, Height};
use crate::observer::{ActionKind, EventLevel, Observer};
use etheram_core::types::PeerId;

pub struct NoOpObserver;

impl Observer for NoOpObserver {
    #[inline]
    fn min_level(&self) -> EventLevel {
        EventLevel::None
    }

    #[inline]
    fn node_started(&mut self, _peer_id: PeerId) {}

    #[inline]
    fn message_received(&mut self, _peer_id: PeerId, _source: &MessageSource) {}

    #[inline]
    fn context_built(
        &mut self,
        _peer_id: PeerId,
        _height: Height,
        _state_root: Hash,
        _pending_tx_count: usize,
    ) {
    }

    #[inline]
    fn action_emitted(&mut self, _peer_id: PeerId, _kind: &ActionKind) {}

    #[inline]
    fn mutation_applied(&mut self, _peer_id: PeerId, _kind: &ActionKind) {}

    #[inline]
    fn output_executed(&mut self, _peer_id: PeerId, _kind: &ActionKind) {}

    #[inline]
    fn step_completed(&mut self, _peer_id: PeerId, _processed: bool) {}
}
