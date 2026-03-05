// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Address;
use crate::common_types::{account::Account, block::Block, types::Hash, types::Height};
use crate::execution::transaction_receipt::TransactionReceipt;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub enum StorageQueryResult {
    Account(Option<Account>),
    ContractStorage(Option<Hash>),
    Accounts(BTreeMap<Address, Account>),
    ContractStorageEntries(BTreeMap<(Address, Hash), Hash>),
    Height(Height),
    StateRoot(Hash),
    Block(Option<Block>),
    Receipts(Vec<TransactionReceipt>),
}
