// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::builders::cache_builder::CacheBuilder;
use etheram_node::common_types::cache_adapter::CacheAdapter;
use etheram_node::common_types::transaction::Transaction;

pub fn build_cache() -> Result<Box<dyn CacheAdapter<Key = (), Value = Transaction>>, String> {
    CacheBuilder::default()
        .build()
        .map_err(|error| format!("failed to build cache: {error:?}"))
}
