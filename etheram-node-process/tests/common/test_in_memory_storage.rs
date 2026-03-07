// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::storage::Storage;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::types::Address;
use etheram_node::implementations::in_memory_storage::InMemoryStorage as NodeInMemoryStorage;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use etheram_node_process::infra::storage::sync_storage::SyncStorage;
use std::sync::Arc;
use std::sync::Mutex;

type SharedStorage = Arc<Mutex<NodeInMemoryStorage>>;

#[derive(Clone)]
pub struct TestInMemoryStorage {
    inner: SharedStorage,
}

impl TestInMemoryStorage {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            inner: Arc::new(Mutex::new(NodeInMemoryStorage::new())),
        })
    }

    pub fn apply_synced_blocks(&self, blocks: &[Block]) {
        let mut guard = self.inner.lock().expect("storage lock poisoned");
        for block in blocks {
            guard.mutate(StorageMutation::StoreBlock(block.clone()));
            guard.mutate(StorageMutation::IncrementHeight);
        }
    }
}

impl Storage for TestInMemoryStorage {
    type Key = Address;
    type Mutation = StorageMutation;
    type Query = StorageQuery;
    type QueryResult = StorageQueryResult;
    type Value = Account;

    fn mutate(&mut self, mutation: Self::Mutation) {
        let mut guard = self.inner.lock().expect("storage lock poisoned");
        guard.mutate(mutation);
    }

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        let guard = self.inner.lock().expect("storage lock poisoned");
        guard.query(query)
    }
}

impl SyncStorage for TestInMemoryStorage {
    fn apply_synced_blocks(&self, blocks: &[Block]) {
        self.apply_synced_blocks(blocks);
    }
}
