// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::transaction::Transaction;
use super::types::{Gas, Hash, Height};
use alloc::vec::Vec;
use etheram_core::types::PeerId;

pub const BLOCK_GAS_LIMIT: Gas = 10_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub height: Height,
    pub proposer: PeerId,
    pub transactions: Vec<Transaction>,
    pub state_root: Hash,
    pub post_state_root: Hash,
    pub receipts_root: Hash,
    pub gas_limit: Gas,
}

impl Block {
    pub fn new(
        height: Height,
        proposer: PeerId,
        transactions: Vec<Transaction>,
        state_root: Hash,
        gas_limit: Gas,
    ) -> Self {
        Self {
            height,
            proposer,
            transactions,
            state_root,
            post_state_root: [0u8; 32],
            receipts_root: [0u8; 32],
            gas_limit,
        }
    }

    pub fn empty(height: Height, proposer: PeerId, state_root: Hash) -> Self {
        Self::new(height, proposer, Vec::new(), state_root, BLOCK_GAS_LIMIT)
    }

    pub fn compute_hash(&self) -> Hash {
        let mut hash = [0u8; 32];
        let height_bytes = self.height.to_le_bytes();
        let proposer_bytes = self.proposer.to_le_bytes();
        hash[0..8].copy_from_slice(&height_bytes);
        hash[8..16].copy_from_slice(&proposer_bytes);
        for (i, b) in self.state_root.iter().enumerate() {
            hash[i] ^= b;
        }
        for (tx_idx, tx) in self.transactions.iter().enumerate() {
            let position = (tx_idx as u8).wrapping_add(1);
            for (i, b) in tx.from.iter().enumerate() {
                hash[i % 32] ^= b.wrapping_mul(position);
            }
            for (i, b) in tx.to.iter().enumerate() {
                hash[(i + 20) % 32] ^= b.wrapping_mul(position);
            }
            for (i, b) in tx.value.to_le_bytes().iter().enumerate() {
                hash[(i + 8) % 32] ^= b.wrapping_mul(position);
            }
            for (i, b) in tx.gas_limit.to_le_bytes().iter().enumerate() {
                hash[(i + 16) % 32] ^= b.wrapping_mul(position);
            }
            for (i, b) in tx.nonce.to_le_bytes().iter().enumerate() {
                hash[(i + 24) % 32] ^= b.wrapping_mul(position);
            }
        }
        for (i, b) in self.post_state_root.iter().enumerate() {
            hash[(i + 3) % 32] ^= *b;
        }
        for (i, b) in self.receipts_root.iter().enumerate() {
            hash[(i + 7) % 32] ^= *b;
        }
        for (i, b) in self.gas_limit.to_le_bytes().iter().enumerate() {
            hash[(i + 11) % 32] ^= b;
        }
        hash
    }
}
