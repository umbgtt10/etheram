// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::codec::block_codec::BlockCodec;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Balance;
use etheram_node::common_types::types::Gas;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::common_types::types::Nonce;
use etheram_node::execution::transaction_receipt::TransactionReceipt;
use etheram_node::execution::transaction_result::TransactionStatus;
use serde::Deserialize;
use serde::Serialize;
use std::vec::Vec;

pub struct StorageCodec;

impl StorageCodec {
    pub fn decode_account(bytes: &[u8]) -> Result<Account, String> {
        postcard::from_bytes::<WireAccount>(bytes)
            .map(Account::from)
            .map_err(|error| format!("failed to decode account: {error}"))
    }

    pub fn decode_address(bytes: &[u8]) -> Result<Address, String> {
        bytes
            .try_into()
            .map_err(|_| format!("invalid address length: {}", bytes.len()))
    }

    pub fn decode_block(bytes: &[u8]) -> Result<Block, String> {
        BlockCodec::deserialize(bytes).map_err(|error| format!("failed to decode block: {error}"))
    }

    pub fn decode_contract_storage_key(bytes: &[u8]) -> Result<(Address, Hash), String> {
        if bytes.len() != 52 {
            return Err(format!(
                "invalid contract storage key length: {}",
                bytes.len()
            ));
        }

        let address = Self::decode_address(&bytes[..20])?;
        let slot = Self::decode_hash(&bytes[20..])?;
        Ok((address, slot))
    }

    pub fn decode_hash(bytes: &[u8]) -> Result<Hash, String> {
        bytes
            .try_into()
            .map_err(|_| format!("invalid hash length: {}", bytes.len()))
    }

    pub fn decode_height(bytes: &[u8]) -> Result<Height, String> {
        let raw: [u8; 8] = bytes
            .try_into()
            .map_err(|_| format!("invalid height length: {}", bytes.len()))?;
        Ok(u64::from_be_bytes(raw))
    }

    pub fn decode_receipts(bytes: &[u8]) -> Result<Vec<TransactionReceipt>, String> {
        postcard::from_bytes::<Vec<WireTransactionReceipt>>(bytes)
            .map(|receipts| receipts.into_iter().map(TransactionReceipt::from).collect())
            .map_err(|error| format!("failed to decode receipts: {error}"))
    }

    pub fn encode_account(account: &Account) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&WireAccount::from(account.clone()))
            .map_err(|error| format!("failed to encode account: {error}"))
    }

    pub fn encode_account_key(address: &Address) -> Vec<u8> {
        address.to_vec()
    }

    pub fn encode_block(block: &Block) -> Result<Vec<u8>, String> {
        BlockCodec::serialize(block).map_err(|error| format!("failed to encode block: {error}"))
    }

    pub fn encode_contract_storage_key(address: &Address, slot: &Hash) -> Vec<u8> {
        let mut key = Vec::with_capacity(52);
        key.extend_from_slice(address);
        key.extend_from_slice(slot);
        key
    }

    pub fn encode_hash(hash: &Hash) -> Vec<u8> {
        hash.to_vec()
    }

    pub fn encode_height(height: Height) -> [u8; 8] {
        height.to_be_bytes()
    }

    pub fn encode_receipts(receipts: &[TransactionReceipt]) -> Result<Vec<u8>, String> {
        let wire: Vec<WireTransactionReceipt> = receipts
            .iter()
            .copied()
            .map(WireTransactionReceipt::from)
            .collect();
        postcard::to_allocvec(&wire).map_err(|error| format!("failed to encode receipts: {error}"))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WireAccount {
    balance: Balance,
    nonce: Nonce,
}

impl From<Account> for WireAccount {
    fn from(account: Account) -> Self {
        Self {
            balance: account.balance,
            nonce: account.nonce,
        }
    }
}

impl From<WireAccount> for Account {
    fn from(wire: WireAccount) -> Self {
        Self {
            balance: wire.balance,
            nonce: wire.nonce,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum WireTransactionStatus {
    InvalidOpcode,
    OutOfGas,
    Reverted,
    Success,
}

impl From<TransactionStatus> for WireTransactionStatus {
    fn from(status: TransactionStatus) -> Self {
        match status {
            TransactionStatus::Success => Self::Success,
            TransactionStatus::OutOfGas => Self::OutOfGas,
            TransactionStatus::Reverted => Self::Reverted,
            TransactionStatus::InvalidOpcode => Self::InvalidOpcode,
        }
    }
}

impl From<WireTransactionStatus> for TransactionStatus {
    fn from(wire: WireTransactionStatus) -> Self {
        match wire {
            WireTransactionStatus::Success => Self::Success,
            WireTransactionStatus::OutOfGas => Self::OutOfGas,
            WireTransactionStatus::Reverted => Self::Reverted,
            WireTransactionStatus::InvalidOpcode => Self::InvalidOpcode,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct WireTransactionReceipt {
    cumulative_gas_used: Gas,
    gas_used: Gas,
    status: WireTransactionStatus,
}

impl From<TransactionReceipt> for WireTransactionReceipt {
    fn from(receipt: TransactionReceipt) -> Self {
        Self {
            cumulative_gas_used: receipt.cumulative_gas_used,
            gas_used: receipt.gas_used,
            status: WireTransactionStatus::from(receipt.status),
        }
    }
}

impl From<WireTransactionReceipt> for TransactionReceipt {
    fn from(wire: WireTransactionReceipt) -> Self {
        Self {
            cumulative_gas_used: wire.cumulative_gas_used,
            gas_used: wire.gas_used,
            status: TransactionStatus::from(wire.status),
        }
    }
}
