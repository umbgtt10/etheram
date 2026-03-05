// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::Action;
use crate::collections::action_collection::ActionCollection;
use crate::common_types::account::Account;
use crate::common_types::block::Block;
use crate::common_types::cache_adapter::CacheAdapter;
use crate::common_types::storage_adapter::StorageAdapter;
use crate::common_types::transaction::Transaction;
use crate::common_types::types::{Address, Hash, Height};
use crate::execution::transaction_receipt::TransactionReceipt;
use crate::state::cache::cache_query::CacheQuery;
use crate::state::cache::cache_query_result::CacheQueryResult;
use crate::state::storage::storage_mutation::StorageMutation;
use crate::state::storage::storage_query::StorageQuery;
use crate::state::storage::storage_query_result::StorageQueryResult;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub struct EtheramState {
    storage: Box<dyn StorageAdapter<Key = Address, Value = Account>>,
    cache: Box<dyn CacheAdapter<Key = (), Value = Transaction>>,
}

impl EtheramState {
    pub fn new(
        storage: Box<dyn StorageAdapter<Key = Address, Value = Account>>,
        cache: Box<dyn CacheAdapter<Key = (), Value = Transaction>>,
    ) -> Self {
        Self { storage, cache }
    }

    pub fn query_height(&self) -> u64 {
        match self.storage.query(StorageQuery::GetHeight) {
            StorageQueryResult::Height(h) => h,
            _ => 0,
        }
    }

    pub fn query_account(&self, address: Address) -> Option<Account> {
        match self.storage.query(StorageQuery::GetAccount(address)) {
            StorageQueryResult::Account(account) => account,
            _ => None,
        }
    }

    pub fn query_contract_storage(&self, address: Address, slot: Hash) -> Option<Hash> {
        match self
            .storage
            .query(StorageQuery::GetContractStorage { address, slot })
        {
            StorageQueryResult::ContractStorage(value) => value,
            _ => None,
        }
    }

    pub fn query_block(&self, height: Height) -> Option<Block> {
        match self.storage.query(StorageQuery::GetBlock(height)) {
            StorageQueryResult::Block(block) => block,
            _ => None,
        }
    }

    pub fn query_state_root(&self) -> Hash {
        match self.storage.query(StorageQuery::GetStateRoot) {
            StorageQueryResult::StateRoot(root) => root,
            _ => [0u8; 32],
        }
    }

    pub fn query_pending(&self) -> Vec<Transaction> {
        let CacheQueryResult::Pending(txs) = self.cache.query(CacheQuery::GetPending);
        txs
    }

    pub fn snapshot_accounts(&self) -> BTreeMap<Address, Account> {
        match self.storage.query(StorageQuery::GetAllAccounts) {
            StorageQueryResult::Accounts(accounts) => accounts,
            _ => BTreeMap::new(),
        }
    }

    pub fn snapshot_contract_storage(&self) -> BTreeMap<(Address, Hash), Hash> {
        match self.storage.query(StorageQuery::GetAllContractStorage) {
            StorageQueryResult::ContractStorageEntries(entries) => entries,
            _ => BTreeMap::new(),
        }
    }

    pub fn query_receipts(&self, height: Height) -> Vec<TransactionReceipt> {
        match self.storage.query(StorageQuery::GetReceipts(height)) {
            StorageQueryResult::Receipts(receipts) => receipts,
            _ => Vec::new(),
        }
    }

    pub fn apply_single_mutation(&mut self, mutation: StorageMutation) {
        self.storage.mutate(mutation);
    }

    pub fn apply_mutations<M: Clone>(&mut self, mutations: &ActionCollection<Action<M>>) {
        for action in mutations {
            match action {
                Action::UpdateAccount { address, account } => {
                    self.storage
                        .mutate(StorageMutation::UpdateAccount(*address, account.clone()));
                }
                Action::IncrementHeight => {
                    self.storage.mutate(StorageMutation::IncrementHeight);
                }
                Action::StoreBlock { block } => {
                    self.storage
                        .mutate(StorageMutation::StoreBlock(block.clone()));
                }
                Action::UpdateCache { update } => {
                    self.cache.update(update.clone());
                }
                Action::ExecuteBlock { .. } => {}
                _ => {}
            }
        }
    }
}
