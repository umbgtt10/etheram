// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::types::{Address, Balance, Gas, Nonce};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: Balance,
    pub gas_limit: Gas,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        value: Balance,
        gas_limit: Gas,
        nonce: Nonce,
        data: Vec<u8>,
    ) -> Self {
        Self {
            from,
            to,
            value,
            gas_limit,
            nonce,
            data,
        }
    }

    pub fn transfer(
        from: Address,
        to: Address,
        value: Balance,
        gas_limit: Gas,
        nonce: Nonce,
    ) -> Self {
        Self::new(from, to, value, gas_limit, nonce, Vec::new())
    }
}
