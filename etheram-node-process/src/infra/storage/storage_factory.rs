// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::builders::storage_builder::StorageBuilder;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::storage_adapter::StorageAdapter;
use etheram_node::common_types::types::Address;

pub fn build_storage() -> Result<Box<dyn StorageAdapter<Key = Address, Value = Account>>, String> {
    StorageBuilder::default()
        .build()
        .map_err(|error| format!("failed to build storage: {error:?}"))
}
