// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Hash;
use crate::common_types::types::Height;
use crate::common_types::{account::Account, block::Block, types::Address};
use crate::execution::transaction_receipt::TransactionReceipt;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub enum StorageMutation {
    UpdateAccount(Address, Account),
    UpdateContractStorage {
        address: Address,
        slot: Hash,
        value: Hash,
    },
    IncrementHeight,
    StoreBlock(Block),
    StoreReceipts(Height, Vec<TransactionReceipt>),
}
