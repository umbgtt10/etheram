// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use etheram_node::common_types::types::Height;
use std::collections::BTreeMap;

pub struct SyncState {
    observed_heights: BTreeMap<PeerId, Height>,
    in_flight_request: Option<(PeerId, Height)>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            observed_heights: BTreeMap::new(),
            in_flight_request: None,
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

    pub fn next_request(
        &mut self,
        local_height: Height,
        max_blocks: u64,
    ) -> Option<(PeerId, Height, u64)> {
        if self.in_flight_request.is_some() {
            return None;
        }

        let best_peer = self
            .observed_heights
            .iter()
            .max_by_key(|(_, height)| *height)
            .and_then(|(peer_id, height)| {
                if *height > local_height {
                    Some(*peer_id)
                } else {
                    None
                }
            })?;

        self.in_flight_request = Some((best_peer, local_height));
        Some((best_peer, local_height, max_blocks))
    }

    pub fn complete_in_flight_request(&mut self, peer_id: PeerId, start_height: Height) -> bool {
        match self.in_flight_request {
            Some((expected_peer, expected_height))
                if expected_peer == peer_id && expected_height == start_height =>
            {
                self.in_flight_request = None;
                true
            }
            _ => false,
        }
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}
