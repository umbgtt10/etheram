// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::account::Account;
use crate::common_types::storage_adapter::StorageAdapter;
use crate::common_types::types::Address;
use crate::implementations::in_memory_storage::InMemoryStorage;
use crate::variants::StorageVariant;
use alloc::boxed::Box;
use etheram_core::node_common::build_error::BuildError;

pub struct StorageBuilder {
    storage: Option<Box<dyn StorageAdapter<Key = Address, Value = Account>>>,
}

impl StorageBuilder {
    pub fn new() -> Self {
        Self { storage: None }
    }

    pub fn with_variant(mut self, variant: StorageVariant) -> Self {
        let storage = match variant {
            StorageVariant::InMemory => Box::new(InMemoryStorage::new()),
            StorageVariant::Custom(custom) => custom,
        };
        self.storage = Some(storage);
        self
    }

    pub fn build(
        self,
    ) -> Result<Box<dyn StorageAdapter<Key = Address, Value = Account>>, BuildError> {
        self.storage.ok_or(BuildError::MissingComponent("storage"))
    }
}

impl Default for StorageBuilder {
    fn default() -> Self {
        Self {
            storage: Some(Box::new(InMemoryStorage::new())),
        }
    }
}
