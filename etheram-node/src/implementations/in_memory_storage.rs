// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::state_root::compute_state_root_with_contract_storage;
use crate::common_types::types::{Address, Hash, Height};
use crate::execution::transaction_receipt::TransactionReceipt;
use crate::state::storage::storage_mutation::StorageMutation;
use crate::state::storage::storage_query::StorageQuery;
use crate::state::storage::storage_query_result::StorageQueryResult;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::storage::Storage;

pub struct InMemoryStorage {
    accounts: BTreeMap<Address, Account>,
    contract_storage: BTreeMap<(Address, Hash), Hash>,
    height: Height,
    state_root: Hash,
    blocks: Vec<Block>,
    receipts: BTreeMap<Height, Vec<TransactionReceipt>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
            contract_storage: BTreeMap::new(),
            height: 0,
            state_root: [0u8; 32],
            blocks: Vec::new(),
            receipts: BTreeMap::new(),
        }
    }

    pub fn with_genesis_account(mut self, address: Address, balance: u128) -> Self {
        self.accounts.insert(address, Account::new(balance));
        self.state_root =
            compute_state_root_with_contract_storage(&self.accounts, &self.contract_storage);
        self
    }
}

impl Storage for InMemoryStorage {
    type Query = StorageQuery;
    type Mutation = StorageMutation;
    type Key = Address;
    type Value = Account;
    type QueryResult = StorageQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        match query {
            StorageQuery::GetAccount(addr) => {
                StorageQueryResult::Account(self.accounts.get(&addr).cloned())
            }
            StorageQuery::GetContractStorage { address, slot } => {
                StorageQueryResult::ContractStorage(
                    self.contract_storage.get(&(address, slot)).copied(),
                )
            }
            StorageQuery::GetAllAccounts => StorageQueryResult::Accounts(self.accounts.clone()),
            StorageQuery::GetAllContractStorage => {
                StorageQueryResult::ContractStorageEntries(self.contract_storage.clone())
            }
            StorageQuery::GetHeight => StorageQueryResult::Height(self.height),
            StorageQuery::GetStateRoot => StorageQueryResult::StateRoot(self.state_root),
            StorageQuery::GetBlock(height) => {
                StorageQueryResult::Block(self.blocks.get(height as usize).cloned())
            }
            StorageQuery::GetReceipts(height) => StorageQueryResult::Receipts(
                self.receipts.get(&height).cloned().unwrap_or_default(),
            ),
        }
    }

    fn mutate(&mut self, mutation: Self::Mutation) {
        match mutation {
            StorageMutation::UpdateAccount(addr, account) => {
                self.accounts.insert(addr, account);
                self.state_root = compute_state_root_with_contract_storage(
                    &self.accounts,
                    &self.contract_storage,
                );
            }
            StorageMutation::UpdateContractStorage {
                address,
                slot,
                value,
            } => {
                self.contract_storage.insert((address, slot), value);
                self.state_root = compute_state_root_with_contract_storage(
                    &self.accounts,
                    &self.contract_storage,
                );
            }
            StorageMutation::IncrementHeight => {
                self.height += 1;
            }
            StorageMutation::StoreBlock(block) => {
                self.blocks.push(block);
            }
            StorageMutation::StoreReceipts(height, recs) => {
                self.receipts.insert(height, recs);
            }
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}
