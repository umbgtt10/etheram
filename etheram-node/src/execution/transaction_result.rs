// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Address;
use crate::common_types::types::Gas;
use crate::state::storage::storage_mutation::StorageMutation;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Success,
    OutOfGas,
}

pub struct TransactionResult {
    pub from: Address,
    pub status: TransactionStatus,
    pub gas_used: Gas,
    pub mutations: Vec<StorageMutation>,
}
