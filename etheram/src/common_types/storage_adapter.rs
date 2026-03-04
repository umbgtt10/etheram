// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::types::Address;
use crate::state::storage::storage_mutation::StorageMutation;
use crate::state::storage::storage_query::StorageQuery;
use crate::state::storage::storage_query_result::StorageQueryResult;
use alloc::boxed::Box;
use etheram_core::storage::Storage;

pub trait StorageAdapter:
    Storage<Query = StorageQuery, QueryResult = StorageQueryResult, Mutation = StorageMutation>
{
}

impl<T> StorageAdapter for T where
    T: Storage<Query = StorageQuery, QueryResult = StorageQueryResult, Mutation = StorageMutation>
{
}

impl Storage for Box<dyn StorageAdapter<Key = Address, Value = Account>> {
    type Key = Address;
    type Value = Account;
    type Query = StorageQuery;
    type Mutation = StorageMutation;
    type QueryResult = StorageQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        (**self).query(query)
    }

    fn mutate(&mut self, mutation: Self::Mutation) {
        (**self).mutate(mutation)
    }
}
