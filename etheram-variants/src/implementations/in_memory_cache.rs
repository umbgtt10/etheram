// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram::common_types::transaction::Transaction;
use etheram::state::cache::cache_query::CacheQuery;
use etheram::state::cache::cache_query_result::CacheQueryResult;
use etheram::state::cache::cache_update::CacheUpdate;
use etheram_core::cache::Cache;

pub struct InMemoryCache {
    pending_txs: Vec<Transaction>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            pending_txs: Vec::new(),
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
            CacheQuery::GetPending => CacheQueryResult::Pending(self.pending_txs.clone()),
        }
    }

    fn update(&mut self, update: Self::Update) {
        match update {
            CacheUpdate::AddPending(tx) => {
                self.pending_txs.push(tx);
            }
            CacheUpdate::RemovePending(tx) => {
                self.pending_txs.retain(|t| t != &tx);
            }
            CacheUpdate::ClearPending => {
                self.pending_txs.clear();
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
