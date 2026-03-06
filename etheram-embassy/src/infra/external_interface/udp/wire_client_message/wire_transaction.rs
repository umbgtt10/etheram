// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::{Address, Balance, Gas, GasPrice, Nonce};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WireTransaction {
    pub from: Address,
    pub to: Address,
    pub value: Balance,
    pub gas_limit: Gas,
    pub gas_price: GasPrice,
    pub nonce: Nonce,
    pub data: Vec<u8>,
}

impl From<Transaction> for WireTransaction {
    fn from(tx: Transaction) -> Self {
        Self {
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_limit: tx.gas_limit,
            gas_price: tx.gas_price,
            nonce: tx.nonce,
            data: tx.data,
        }
    }
}

impl From<WireTransaction> for Transaction {
    fn from(wire: WireTransaction) -> Self {
        Transaction::new(
            wire.from,
            wire.to,
            wire.value,
            wire.gas_limit,
            wire.gas_price,
            wire.nonce,
            wire.data,
        )
    }
}
