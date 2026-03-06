// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_message::SyncMessage;
use etheram_core::types::PeerId;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::Mutex;

type SyncQueue = VecDeque<(PeerId, SyncMessage)>;
type SyncQueueState = BTreeMap<PeerId, SyncQueue>;

pub struct SyncBus {
    inbound: Mutex<SyncQueueState>,
}

impl SyncBus {
    pub fn new() -> Self {
        Self {
            inbound: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn enqueue_sync_for(&self, node_id: PeerId, from_peer: PeerId, message: SyncMessage) {
        let mut guard = self.inbound.lock().expect("sync inbound lock poisoned");
        let queue = guard.entry(node_id).or_default();
        queue.push_back((from_peer, message));
    }

    pub fn dequeue_sync_for(&self, node_id: PeerId) -> Option<(PeerId, SyncMessage)> {
        let mut guard = self.inbound.lock().expect("sync inbound lock poisoned");
        let queue = guard.entry(node_id).or_default();
        queue.pop_front()
    }
}

impl Default for SyncBus {
    fn default() -> Self {
        Self::new()
    }
}
