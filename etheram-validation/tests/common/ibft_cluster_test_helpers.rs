// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::implementations::ibft::{
    ibft_message::IbftMessage, signature_scheme::SignatureBytes,
};
use etheram::common_types::block::Block;

fn sequence(height: u64, round: u64, phase: u64) -> u64 {
    (height * 100) + (round * 10) + phase
}

pub fn block(height: u64, proposer: u64) -> Block {
    Block::new(height, proposer, vec![], [0u8; 32])
}

pub fn block_hash(block: &Block) -> [u8; 32] {
    block.compute_hash()
}

pub fn pre_prepare(height: u64, round: u64, block: &Block) -> IbftMessage {
    IbftMessage::PrePrepare {
        sequence: sequence(height, round, 1),
        height,
        round,
        block: block.clone(),
    }
}

pub fn prepare(height: u64, round: u64, block_hash: [u8; 32]) -> IbftMessage {
    IbftMessage::Prepare {
        sequence: sequence(height, round, 2),
        height,
        round,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    }
}

pub fn commit(height: u64, round: u64, block_hash: [u8; 32]) -> IbftMessage {
    IbftMessage::Commit {
        sequence: sequence(height, round, 3),
        height,
        round,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    }
}

pub fn validators() -> Vec<u64> {
    vec![0, 1, 2, 3]
}
