// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait Cache {
    type Key;
    type Value;
    type Query;
    type Update;
    type QueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult;
    fn update(&mut self, update: Self::Update);
    fn invalidate(&mut self, key: Self::Key);
}
