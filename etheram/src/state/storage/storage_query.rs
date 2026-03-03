// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::{Address, Hash, Height};

#[derive(Debug, Clone)]
pub enum StorageQuery {
    GetAccount(Address),
    GetContractStorage { address: Address, slot: Hash },
    GetAllAccounts,
    GetAllContractStorage,
    GetHeight,
    GetStateRoot,
    GetBlock(Height),
    GetReceipts(Height),
}
