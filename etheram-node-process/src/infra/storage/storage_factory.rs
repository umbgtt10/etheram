// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::injected_storage::InjectedStorage;
use crate::infra::storage::injected_storage::InjectedStorageHandle;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::storage_adapter::StorageAdapter;
use etheram_node::common_types::types::Address;

pub type StorageBuildOutput = (
    Box<dyn StorageAdapter<Key = Address, Value = Account>>,
    InjectedStorageHandle,
);

pub fn build_storage() -> Result<StorageBuildOutput, String> {
    let storage = InjectedStorage::new()?;
    let handle = storage.handle();
    let adapter: Box<dyn StorageAdapter<Key = Address, Value = Account>> = Box::new(storage);
    Ok((adapter, handle))
}
