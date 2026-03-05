// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::cache_adapter::CacheAdapter;
use crate::common_types::transaction::Transaction;
use crate::implementations::in_memory_cache::InMemoryCache;
use crate::variants::CacheVariant;
use alloc::boxed::Box;
use etheram_core::node_common::build_error::BuildError;

pub struct CacheBuilder {
    cache: Option<Box<dyn CacheAdapter<Key = (), Value = Transaction>>>,
}

impl CacheBuilder {
    pub fn new() -> Self {
        Self { cache: None }
    }

    pub fn with_variant(mut self, variant: CacheVariant) -> Self {
        let cache = match variant {
            CacheVariant::InMemory => Box::new(InMemoryCache::new()),
            CacheVariant::Custom(custom) => custom,
        };
        self.cache = Some(cache);
        self
    }

    pub fn build(self) -> Result<Box<dyn CacheAdapter<Key = (), Value = Transaction>>, BuildError> {
        self.cache.ok_or(BuildError::MissingComponent("cache"))
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self {
            cache: Some(Box::new(InMemoryCache::new())),
        }
    }
}
