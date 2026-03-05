// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Height;
use alloc::vec::Vec;
use etheram_core::types::PeerId;

pub struct ValidatorSet {
    validators: Vec<PeerId>,
}

impl ValidatorSet {
    pub fn new(validators: Vec<PeerId>) -> Self {
        Self { validators }
    }

    pub fn quorum_size(&self) -> usize {
        2 * self.validators.len() / 3 + 1
    }

    pub fn count(&self) -> usize {
        self.validators.len()
    }

    pub fn validators(&self) -> Vec<PeerId> {
        self.validators.clone()
    }

    pub fn contains(&self, peer_id: PeerId) -> bool {
        self.validators.contains(&peer_id)
    }

    pub fn get_proposer(&self, height: Height) -> PeerId {
        self.validators[(height as usize) % self.validators.len()]
    }

    pub fn get_proposer_for_round(&self, height: Height, round: u64) -> PeerId {
        let index = ((height + round) as usize) % self.validators.len();
        self.validators[index]
    }
}
