// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::state_root::compute_state_root_with_contract_storage;
use etheram::common_types::types::Address;
use etheram::common_types::types::Hash;
use etheram::common_types::types::Height;
use etheram::execution::transaction_receipt::TransactionReceipt;
use etheram::state::storage::storage_mutation::StorageMutation;
use etheram::state::storage::storage_query::StorageQuery;
use etheram::state::storage::storage_query_result::StorageQueryResult;
use etheram_core::storage::Storage;
use etheram_core::types::PeerId;

pub struct SemihostingStorage {
    node_id: PeerId,
    height: Height,
    state_root: Hash,
    blocks: Vec<Block>,
    accounts: BTreeMap<Address, Account>,
    contract_storage: BTreeMap<(Address, Hash), Hash>,
    receipts: BTreeMap<Height, Vec<TransactionReceipt>>,
    mutation_count: u64,
}

impl SemihostingStorage {
    pub fn new(node_id: PeerId) -> Self {
        Self {
            node_id,
            height: 0,
            state_root: [0u8; 32],
            blocks: Vec::new(),
            accounts: BTreeMap::new(),
            contract_storage: BTreeMap::new(),
            receipts: BTreeMap::new(),
            mutation_count: 0,
        }
    }

    pub fn with_genesis_account(mut self, address: Address, balance: u128) -> Self {
        self.accounts
            .entry(address)
            .or_insert_with(|| Account::new(balance));
        self.state_root =
            compute_state_root_with_contract_storage(&self.accounts, &self.contract_storage);
        self
    }

    fn metadata_path(&self) -> Vec<u8> {
        format!("persistency/etheram_node_{}_metadata.bin\0", self.node_id).into_bytes()
    }

    fn blocks_path(&self) -> Vec<u8> {
        format!("persistency/etheram_node_{}_blocks.bin\0", self.node_id).into_bytes()
    }

    fn accounts_path(&self) -> Vec<u8> {
        format!("persistency/etheram_node_{}_accounts.bin\0", self.node_id).into_bytes()
    }

    fn persist_metadata(&self) {
        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&self.height.to_le_bytes());
        data.extend_from_slice(&self.state_root);
        let _ = self.write_file(&self.metadata_path(), &data);
    }

    fn persist_blocks(&self) {
        let data = Self::serialize_blocks(&self.blocks);
        let _ = self.write_file(&self.blocks_path(), &data);
    }

    fn persist_accounts(&self) {
        let data = Self::serialize_accounts(&self.accounts);
        let _ = self.write_file(&self.accounts_path(), &data);
    }

    fn serialize_blocks(blocks: &[Block]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(blocks.len() as u64).to_le_bytes());
        for block in blocks {
            data.extend_from_slice(&block.proposer.to_le_bytes());
            data.extend_from_slice(&block.state_root);
            data.extend_from_slice(&(block.transactions.len() as u32).to_le_bytes());
            for tx in &block.transactions {
                data.extend_from_slice(&tx.from);
                data.extend_from_slice(&tx.to);
                data.extend_from_slice(&tx.value.to_le_bytes());
                data.extend_from_slice(&tx.gas_limit.to_le_bytes());
                data.extend_from_slice(&tx.nonce.to_le_bytes());
            }
        }
        data
    }

    fn serialize_accounts(accounts: &BTreeMap<Address, Account>) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(accounts.len() as u64).to_le_bytes());
        for (addr, account) in accounts {
            data.extend_from_slice(addr);
            data.extend_from_slice(&account.balance.to_le_bytes());
            data.extend_from_slice(&account.nonce.to_le_bytes());
        }
        data
    }

    fn write_file(&self, path: &[u8], data: &[u8]) -> Result<(), ()> {
        unsafe {
            let mode: usize = 0x0000_0006;
            let fd = cortex_m_semihosting::syscall!(OPEN, path.as_ptr(), mode, path.len() - 1);
            if fd == usize::MAX {
                return Err(());
            }
            let result = cortex_m_semihosting::syscall!(WRITE, fd, data.as_ptr(), data.len());
            cortex_m_semihosting::syscall!(CLOSE, fd);
            if result == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

impl Storage for SemihostingStorage {
    type Key = Address;
    type Value = Account;
    type Query = StorageQuery;
    type Mutation = StorageMutation;
    type QueryResult = StorageQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        match query {
            StorageQuery::GetHeight => StorageQueryResult::Height(self.height),
            StorageQuery::GetStateRoot => StorageQueryResult::StateRoot(self.state_root),
            StorageQuery::GetBlock(height) => {
                StorageQueryResult::Block(self.blocks.get(height as usize).cloned())
            }
            StorageQuery::GetAccount(addr) => {
                StorageQueryResult::Account(self.accounts.get(&addr).cloned())
            }
            StorageQuery::GetContractStorage { address, slot } => {
                StorageQueryResult::ContractStorage(
                    self.contract_storage.get(&(address, slot)).copied(),
                )
            }
            StorageQuery::GetAllAccounts => StorageQueryResult::Accounts(self.accounts.clone()),
            StorageQuery::GetAllContractStorage => {
                StorageQueryResult::ContractStorageEntries(self.contract_storage.clone())
            }
            StorageQuery::GetReceipts(height) => StorageQueryResult::Receipts(
                self.receipts.get(&height).cloned().unwrap_or_default(),
            ),
        }
    }

    fn mutate(&mut self, mutation: Self::Mutation) {
        self.mutation_count += 1;
        crate::info!(
            "SemihostingStorage mutation #{}: {:?}",
            self.mutation_count,
            mutation
        );
        match mutation {
            StorageMutation::IncrementHeight => {
                self.height += 1;
                self.persist_metadata();
            }
            StorageMutation::StoreBlock(block) => {
                self.blocks.push(block);
                self.persist_blocks();
            }
            StorageMutation::UpdateContractStorage {
                address,
                slot,
                value,
            } => {
                self.contract_storage.insert((address, slot), value);
                self.state_root = compute_state_root_with_contract_storage(
                    &self.accounts,
                    &self.contract_storage,
                );
                self.persist_metadata();
            }
            StorageMutation::UpdateAccount(addr, account) => {
                self.accounts.insert(addr, account);
                self.persist_accounts();
                self.state_root = compute_state_root_with_contract_storage(
                    &self.accounts,
                    &self.contract_storage,
                );
                self.persist_metadata();
            }
            StorageMutation::StoreReceipts(height, recs) => {
                self.receipts.insert(height, recs);
            }
        }
    }
}
