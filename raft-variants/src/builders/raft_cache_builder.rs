// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::in_memory_raft_cache::InMemoryRaftCache;
use crate::variants::RaftCacheVariant;
use alloc::boxed::Box;
use raft_node::common_types::cache_adapter::CacheAdapter;

pub struct RaftCacheBuilder {
    cache: Option<Box<dyn CacheAdapter<Key = (), Value = ()>>>,
}

impl RaftCacheBuilder {
    pub fn new() -> Self {
        Self { cache: None }
    }

    pub fn with_variant(mut self, variant: RaftCacheVariant) -> Self {
        let cache: Box<dyn CacheAdapter<Key = (), Value = ()>> = match variant {
            RaftCacheVariant::InMemory => Box::new(InMemoryRaftCache::new()),
            RaftCacheVariant::Custom(custom) => custom,
        };
        self.cache = Some(cache);
        self
    }

    pub fn build(self) -> Result<Box<dyn CacheAdapter<Key = (), Value = ()>>, BuildError> {
        self.cache.ok_or(BuildError::MissingComponent("cache"))
    }
}

impl Default for RaftCacheBuilder {
    fn default() -> Self {
        Self {
            cache: Some(Box::new(InMemoryRaftCache::new())),
        }
    }
}
