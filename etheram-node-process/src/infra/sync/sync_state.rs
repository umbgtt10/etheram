// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use etheram_node::common_types::types::Height;
use std::collections::BTreeMap;

pub struct SyncState {
    observed_heights: BTreeMap<PeerId, Height>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            observed_heights: BTreeMap::new(),
        }
    }

    pub fn observe_status(&mut self, peer_id: PeerId, height: Height) {
        self.observed_heights.insert(peer_id, height);
    }

    pub fn highest_peer_height(&self) -> Option<Height> {
        self.observed_heights.values().copied().max()
    }

    pub fn lag_distance(&self, local_height: Height) -> Option<Height> {
        self.highest_peer_height().and_then(|remote| {
            if remote > local_height {
                Some(remote - local_height)
            } else {
                None
            }
        })
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}
