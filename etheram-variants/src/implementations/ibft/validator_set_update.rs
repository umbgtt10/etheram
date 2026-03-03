// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use barechain_core::types::PeerId;
use etheram::common_types::types::Height;

#[derive(Clone)]
pub struct ValidatorSetUpdate {
    pub target_height: Height,
    pub validators: Vec<PeerId>,
}

impl ValidatorSetUpdate {
    pub fn new(target_height: Height, validators: Vec<PeerId>) -> Self {
        Self {
            target_height,
            validators,
        }
    }
}
