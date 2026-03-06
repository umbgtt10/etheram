// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::storage::Storage;
use etheram_node::builders::storage_builder::StorageBuilder;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::storage_adapter::StorageAdapter;
use etheram_node::common_types::types::Address;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use std::cell::RefCell;
use std::rc::Rc;

type SharedStorage = Rc<RefCell<Box<dyn StorageAdapter<Key = Address, Value = Account>>>>;

pub struct InjectedStorage {
    inner: SharedStorage,
}

#[derive(Clone)]
pub struct InjectedStorageHandle {
    inner: SharedStorage,
}

impl InjectedStorage {
    pub fn new() -> Result<Self, String> {
        let storage = StorageBuilder::default()
            .build()
            .map_err(|error| format!("failed to build injected storage: {error:?}"))?;
        Ok(Self {
            inner: Rc::new(RefCell::new(storage)),
        })
    }

    pub fn handle(&self) -> InjectedStorageHandle {
        InjectedStorageHandle {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl InjectedStorageHandle {
    pub fn apply_synced_blocks(&self, blocks: &[Block]) {
        let mut guard = self.inner.borrow_mut();
        for block in blocks {
            guard.mutate(StorageMutation::StoreBlock(block.clone()));
            guard.mutate(StorageMutation::IncrementHeight);
        }
    }
}

impl Storage for InjectedStorage {
    type Key = Address;
    type Value = Account;
    type Query = StorageQuery;
    type Mutation = StorageMutation;
    type QueryResult = StorageQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        let guard = self.inner.borrow();
        guard.query(query)
    }

    fn mutate(&mut self, mutation: Self::Mutation) {
        let mut guard = self.inner.borrow_mut();
        guard.mutate(mutation);
    }
}
