// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use etheram_node::common_types::types::{Hash, Height};

pub struct VoteTracker {
    quorum_size: usize,
    votes: BTreeMap<(Height, u64, Hash), BTreeSet<PeerId>>,
}

impl VoteTracker {
    pub fn new(quorum_size: usize) -> Self {
        Self {
            quorum_size,
            votes: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, height: Height, round: u64, block_hash: Hash, voter: PeerId) {
        self.votes
            .entry((height, round, block_hash))
            .or_default()
            .insert(voter);
    }

    pub fn has_quorum(&self, height: Height, round: u64, block_hash: Hash) -> bool {
        self.votes
            .get(&(height, round, block_hash))
            .is_some_and(|voters| voters.len() >= self.quorum_size)
    }

    pub fn voters(&self, height: Height, round: u64, block_hash: Hash) -> Vec<PeerId> {
        self.votes
            .get(&(height, round, block_hash))
            .map(|voters| voters.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn clear(&mut self) {
        self.votes.clear();
    }

    pub fn snapshot(&self) -> BTreeMap<(Height, u64, Hash), Vec<PeerId>> {
        self.votes
            .iter()
            .map(|(key, voters)| (*key, voters.iter().copied().collect()))
            .collect()
    }

    pub fn from_snapshot(
        quorum_size: usize,
        snapshot: BTreeMap<(Height, u64, Hash), Vec<PeerId>>,
    ) -> Self {
        let votes = snapshot
            .into_iter()
            .map(|(key, voters)| (key, voters.into_iter().collect()))
            .collect();
        Self { quorum_size, votes }
    }
}
