// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::sled_storage::SledStorage;

pub fn build_storage(db_path: &str) -> Result<SledStorage, String> {
    SledStorage::new(db_path)
}
