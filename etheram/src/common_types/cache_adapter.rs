// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::transaction::Transaction;
use crate::state::cache::cache_query::CacheQuery;
use crate::state::cache::cache_query_result::CacheQueryResult;
use crate::state::cache::cache_update::CacheUpdate;
use alloc::boxed::Box;
use etheram_core::cache::Cache;

pub trait CacheAdapter:
    Cache<Query = CacheQuery, QueryResult = CacheQueryResult, Update = CacheUpdate>
{
}

impl<T> CacheAdapter for T where
    T: Cache<Query = CacheQuery, QueryResult = CacheQueryResult, Update = CacheUpdate>
{
}

impl Cache for Box<dyn CacheAdapter<Key = (), Value = Transaction>> {
    type Key = ();
    type Value = Transaction;
    type Query = CacheQuery;
    type Update = CacheUpdate;
    type QueryResult = CacheQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        (**self).query(query)
    }

    fn update(&mut self, update: Self::Update) {
        (**self).update(update)
    }

    fn invalidate(&mut self, _key: Self::Key) {}
}
