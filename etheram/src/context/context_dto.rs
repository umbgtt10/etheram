// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::{
    account::Account,
    transaction::Transaction,
    types::{Address, Hash, Height},
};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use barechain_core::types::PeerId;

#[derive(Debug, Clone)]
pub struct Context {
    pub peer_id: PeerId,
    pub current_height: Height,
    pub state_root: Hash,
    pub accounts: BTreeMap<Address, Account>,
    pub pending_txs: Vec<Transaction>,
}

impl Context {
    pub fn new(peer_id: PeerId, current_height: Height, state_root: Hash) -> Self {
        Self {
            peer_id,
            current_height,
            state_root,
            accounts: BTreeMap::new(),
            pending_txs: Vec::new(),
        }
    }
}
