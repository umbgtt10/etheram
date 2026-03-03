// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::signature_scheme::SignatureBytes;
use alloc::vec::Vec;
use barechain_core::types::PeerId;
use etheram::common_types::types::Hash;
use etheram::common_types::types::Height;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedCertificate {
    pub height: Height,
    pub round: u64,
    pub block_hash: Hash,
    pub signed_prepares: Vec<(PeerId, SignatureBytes)>,
}
