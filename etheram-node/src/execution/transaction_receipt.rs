// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Gas;
use crate::execution::transaction_result::TransactionStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionReceipt {
    pub status: TransactionStatus,
    pub gas_used: Gas,
    pub cumulative_gas_used: Gas,
}

pub fn summarize_receipts(receipts: &[TransactionReceipt]) -> (usize, usize, usize, usize) {
    receipts.iter().fold(
        (0usize, 0usize, 0usize, 0usize),
        |(s, o, r, i), receipt| match receipt.status {
            TransactionStatus::Success => (s + 1, o, r, i),
            TransactionStatus::OutOfGas => (s, o + 1, r, i),
            TransactionStatus::Reverted => (s, o, r + 1, i),
            TransactionStatus::InvalidOpcode => (s, o, r, i + 1),
        },
    )
}
