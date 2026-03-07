// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Balance;
use etheram_node::common_types::types::Gas;
use etheram_node::common_types::types::GasPrice;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::common_types::types::Nonce;
use serde::Deserialize;
use serde::Serialize;
use std::vec::Vec;

pub struct BlockCodec;

impl BlockCodec {
    pub fn deserialize(bytes: &[u8]) -> Result<Block, postcard::Error> {
        let wire: WireBlock = postcard::from_bytes(bytes)?;
        Ok(Block::from(wire))
    }

    pub fn serialize(block: &Block) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(&WireBlock::from(block.clone()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WireTransaction {
    from: Address,
    to: Address,
    value: Balance,
    gas_limit: Gas,
    gas_price: GasPrice,
    nonce: Nonce,
    data: Vec<u8>,
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
        Self {
            from: wire.from,
            to: wire.to,
            value: wire.value,
            gas_limit: wire.gas_limit,
            gas_price: wire.gas_price,
            nonce: wire.nonce,
            data: wire.data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WireBlock {
    height: Height,
    proposer: u64,
    transactions: Vec<WireTransaction>,
    state_root: Hash,
    post_state_root: Hash,
    receipts_root: Hash,
    gas_limit: Gas,
}

impl From<Block> for WireBlock {
    fn from(block: Block) -> Self {
        Self {
            height: block.height,
            proposer: block.proposer,
            transactions: block
                .transactions
                .into_iter()
                .map(WireTransaction::from)
                .collect(),
            state_root: block.state_root,
            post_state_root: block.post_state_root,
            receipts_root: block.receipts_root,
            gas_limit: block.gas_limit,
        }
    }
}

impl From<WireBlock> for Block {
    fn from(wire: WireBlock) -> Self {
        Self {
            height: wire.height,
            proposer: wire.proposer,
            transactions: wire
                .transactions
                .into_iter()
                .map(Transaction::from)
                .collect(),
            state_root: wire.state_root,
            post_state_root: wire.post_state_root,
            receipts_root: wire.receipts_root,
            gas_limit: wire.gas_limit,
        }
    }
}
