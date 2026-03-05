// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Hash;
use crate::execution::transaction_result::TransactionResult;
use crate::execution::transaction_result::TransactionStatus;

pub fn compute_receipts_root(results: &[TransactionResult]) -> Hash {
    let mut hash = [0u8; 32];
    for (position, result) in (0_u64..).zip(results.iter()) {
        let mix = position.wrapping_add(1) as u8;
        let status_byte: u8 = match result.status {
            TransactionStatus::Success => 0x01,
            TransactionStatus::OutOfGas => 0x02,
        };
        hash[position as usize % 32] ^= status_byte.wrapping_mul(mix);
        for (i, b) in result.gas_used.to_le_bytes().iter().enumerate() {
            hash[(i + 1) % 32] ^= b.wrapping_mul(mix);
        }
    }
    hash
}
