// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::state::cache::cache_query::RaftCacheQuery;
use crate::state::cache::cache_query_result::RaftCacheQueryResult;
use crate::state::cache::cache_update::RaftCacheUpdate;
use alloc::boxed::Box;
use etheram_core::cache::Cache;

pub trait CacheAdapter:
    Cache<Query = RaftCacheQuery, QueryResult = RaftCacheQueryResult, Update = RaftCacheUpdate>
{
}

impl<T> CacheAdapter for T where
    T: Cache<Query = RaftCacheQuery, QueryResult = RaftCacheQueryResult, Update = RaftCacheUpdate>
{
}

impl Cache for Box<dyn CacheAdapter<Key = (), Value = ()>> {
    type Key = ();
    type Value = ();
    type Query = RaftCacheQuery;
    type Update = RaftCacheUpdate;
    type QueryResult = RaftCacheQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        (**self).query(query)
    }

    fn update(&mut self, update: Self::Update) {
        (**self).update(update);
    }

    fn invalidate(&mut self, _key: Self::Key) {}
}
