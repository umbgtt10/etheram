// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::transaction::Transaction;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub enum CacheQueryResult {
    Pending(Vec<Transaction>),
}
