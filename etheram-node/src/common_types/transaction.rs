// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::types::{Address, Balance, Gas, GasPrice, Nonce};
use alloc::vec::Vec;
use core::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: Balance,
    pub gas_limit: Gas,
    pub gas_price: GasPrice,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gas_price
            .cmp(&other.gas_price)
            .then(other.nonce.cmp(&self.nonce))
            .then(other.from.cmp(&self.from))
            .then(self.to.cmp(&other.to))
            .then(self.value.cmp(&other.value))
            .then(self.gas_limit.cmp(&other.gas_limit))
            .then(self.data.cmp(&other.data))
    }
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        value: Balance,
        gas_limit: Gas,
        gas_price: GasPrice,
        nonce: Nonce,
        data: Vec<u8>,
    ) -> Self {
        Self {
            from,
            to,
            value,
            gas_limit,
            gas_price,
            nonce,
            data,
        }
    }

    pub fn transfer(
        from: Address,
        to: Address,
        value: Balance,
        gas_limit: Gas,
        gas_price: GasPrice,
        nonce: Nonce,
    ) -> Self {
        Self::new(from, to, value, gas_limit, gas_price, nonce, Vec::new())
    }
}
