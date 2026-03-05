// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::block::Block;
use crate::common_types::types::{Hash, Height};
use crate::implementations::ibft::prepared_certificate::PreparedCertificate;
use crate::implementations::ibft::signature_scheme::SignatureBytes;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IbftMessage {
    PrePrepare {
        sequence: u64,
        height: Height,
        round: u64,
        block: Block,
    },
    Prepare {
        sequence: u64,
        height: Height,
        round: u64,
        block_hash: Hash,
        sender_signature: SignatureBytes,
    },
    Commit {
        sequence: u64,
        height: Height,
        round: u64,
        block_hash: Hash,
        sender_signature: SignatureBytes,
    },
    ViewChange {
        sequence: u64,
        height: Height,
        round: u64,
        prepared_certificate: Option<PreparedCertificate>,
    },
    NewView {
        sequence: u64,
        height: Height,
        round: u64,
        prepared_certificate: Option<PreparedCertificate>,
        view_change_senders: Vec<u64>,
    },
}
