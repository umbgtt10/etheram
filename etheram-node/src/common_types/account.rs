// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::types::{Balance, Nonce};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub balance: Balance,

    pub nonce: Nonce,
}

impl Account {
    pub fn new(balance: Balance) -> Self {
        Self { balance, nonce: 0 }
    }

    pub fn empty() -> Self {
        Self::new(0)
    }
}
