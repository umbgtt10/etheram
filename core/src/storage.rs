// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait Storage {
    type Key;
    type Value;
    type Query;
    type Mutation;
    type QueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult;
    fn mutate(&mut self, mutation: Self::Mutation);
}
