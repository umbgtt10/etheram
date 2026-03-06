// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::transaction::Transaction;
use crate::common_types::types::{Address, Nonce};
use crate::state::cache::cache_query::CacheQuery;
use crate::state::cache::cache_query_result::CacheQueryResult;
use crate::state::cache::cache_update::CacheUpdate;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use etheram_core::cache::Cache;

pub const PENDING_TX_POOL_CAPACITY: usize = 4096;

pub struct InMemoryCache {
    pending_txs: BTreeSet<Transaction>,
    pending_index: BTreeMap<(Address, Nonce), Transaction>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            pending_txs: BTreeSet::new(),
            pending_index: BTreeMap::new(),
        }
    }
}

impl Cache for InMemoryCache {
    type Query = CacheQuery;
    type Update = CacheUpdate;
    type Key = ();
    type Value = Transaction;
    type QueryResult = CacheQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        match query {
            CacheQuery::GetPending => CacheQueryResult::Pending(
                self.pending_txs.iter().rev().cloned().collect::<Vec<_>>(),
            ),
        }
    }

    fn update(&mut self, update: Self::Update) {
        match update {
            CacheUpdate::AddPending(tx) => {
                let key = (tx.from, tx.nonce);
                if let Some(existing) = self.pending_index.get(&key).cloned() {
                    if tx.gas_price <= existing.gas_price {
                        return;
                    }
                    self.pending_txs.remove(&existing);
                }
                if self.pending_txs.len() >= PENDING_TX_POOL_CAPACITY {
                    if let Some(lowest) = self.pending_txs.iter().next().cloned() {
                        if tx <= lowest {
                            return;
                        }
                        self.pending_txs.remove(&lowest);
                        self.pending_index.remove(&(lowest.from, lowest.nonce));
                    }
                }
                self.pending_txs.insert(tx.clone());
                self.pending_index.insert(key, tx);
            }
            CacheUpdate::RemovePending(tx) => {
                if let Some(existing) = self.pending_index.remove(&(tx.from, tx.nonce)) {
                    self.pending_txs.remove(&existing);
                }
            }
            CacheUpdate::ClearPending => {
                self.pending_txs.clear();
                self.pending_index.clear();
            }
        }
    }

    fn invalidate(&mut self, _key: Self::Key) {}
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}
