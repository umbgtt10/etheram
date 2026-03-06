// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::in_memory_storage::InMemoryStorage;

pub fn build_storage() -> Result<InMemoryStorage, String> {
    InMemoryStorage::new()
}
