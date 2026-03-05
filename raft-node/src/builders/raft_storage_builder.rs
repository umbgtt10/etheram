// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::common_types::storage_adapter::StorageAdapter;
use crate::implementations::in_memory_raft_storage::InMemoryRaftStorage;
use crate::variants::RaftStorageVariant;
use alloc::boxed::Box;

pub struct RaftStorageBuilder<P: Clone + 'static> {
    storage: Option<Box<dyn StorageAdapter<P, Key = (), Value = ()>>>,
}

impl<P: Clone + 'static> RaftStorageBuilder<P> {
    pub fn new() -> Self {
        Self { storage: None }
    }

    pub fn with_variant(mut self, variant: RaftStorageVariant<P>) -> Self {
        let storage: Box<dyn StorageAdapter<P, Key = (), Value = ()>> = match variant {
            RaftStorageVariant::InMemory => Box::new(InMemoryRaftStorage::new()),
            RaftStorageVariant::Custom(custom) => custom,
        };
        self.storage = Some(storage);
        self
    }

    pub fn build(self) -> Result<Box<dyn StorageAdapter<P, Key = (), Value = ()>>, BuildError> {
        self.storage.ok_or(BuildError::MissingComponent("storage"))
    }
}

impl<P: Clone + 'static> Default for RaftStorageBuilder<P> {
    fn default() -> Self {
        Self {
            storage: Some(Box::new(InMemoryRaftStorage::new())),
        }
    }
}
