// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::state::storage::storage_mutation::RaftStorageMutation;
use crate::state::storage::storage_query::RaftStorageQuery;
use crate::state::storage::storage_query_result::RaftStorageQueryResult;
use alloc::boxed::Box;
use etheram_core::storage::Storage;

pub trait StorageAdapter<P>:
    Storage<
    Query = RaftStorageQuery,
    QueryResult = RaftStorageQueryResult<P>,
    Mutation = RaftStorageMutation<P>,
>
{
}

impl<T, P> StorageAdapter<P> for T where
    T: Storage<
        Query = RaftStorageQuery,
        QueryResult = RaftStorageQueryResult<P>,
        Mutation = RaftStorageMutation<P>,
    >
{
}

impl<P: Clone + 'static> Storage for Box<dyn StorageAdapter<P, Key = (), Value = ()>> {
    type Key = ();
    type Value = ();
    type Query = RaftStorageQuery;
    type Mutation = RaftStorageMutation<P>;
    type QueryResult = RaftStorageQueryResult<P>;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        (**self).query(query)
    }

    fn mutate(&mut self, mutation: Self::Mutation) {
        (**self).mutate(mutation);
    }
}
