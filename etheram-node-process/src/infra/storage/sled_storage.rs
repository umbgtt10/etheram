// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::storage_codec::StorageCodec;
use crate::infra::storage::sync_storage::SyncStorage;
use etheram_core::storage::Storage;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::state_root::compute_state_root_with_contract_storage;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::execution::transaction_receipt::TransactionReceipt;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use sled::open;
use sled::Batch;
use sled::Db;
use sled::Tree;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

const META_HEIGHT_KEY: &[u8] = b"height";
const META_STATE_ROOT_KEY: &[u8] = b"state_root";

type SharedSledStorage = Arc<Mutex<SledStorageInner>>;

#[derive(Clone)]
pub struct SledStorage {
    inner: SharedSledStorage,
}

struct SledStorageInner {
    accounts: Tree,
    blocks: Tree,
    contract_storage: Tree,
    db: Db,
    meta: Tree,
    receipts: Tree,
}

impl SledStorage {
    pub fn new(db_path: &str) -> Result<Self, String> {
        Ok(Self {
            inner: Arc::new(Mutex::new(SledStorageInner::new(db_path)?)),
        })
    }

    pub fn apply_synced_blocks(&self, blocks: &[Block]) {
        let mut guard = self.inner.lock().expect("storage lock poisoned");
        guard.apply_synced_blocks(blocks);
    }
}

impl Storage for SledStorage {
    type Key = Address;
    type Mutation = StorageMutation;
    type Query = StorageQuery;
    type QueryResult = StorageQueryResult;
    type Value = Account;

    fn mutate(&mut self, mutation: Self::Mutation) {
        let mut guard = self.inner.lock().expect("storage lock poisoned");
        guard.mutate(mutation);
    }

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        let guard = self.inner.lock().expect("storage lock poisoned");
        guard.query(query)
    }
}

impl SyncStorage for SledStorage {
    fn apply_synced_blocks(&self, blocks: &[Block]) {
        self.apply_synced_blocks(blocks);
    }
}

impl SledStorageInner {
    fn apply_synced_blocks(&mut self, blocks: &[Block]) {
        if blocks.is_empty() {
            return;
        }

        let mut block_batch = Batch::default();
        let next_height = blocks.last().map(|block| block.height + 1).unwrap_or(0);

        for block in blocks {
            let encoded = StorageCodec::encode_block(block).unwrap_or_else(|error| {
                panic!("failed to encode block {}: {}", block.height, error)
            });
            block_batch.insert(StorageCodec::encode_height(block.height).to_vec(), encoded);
        }

        self.blocks
            .apply_batch(block_batch)
            .unwrap_or_else(|error| panic!("failed to batch store blocks: {}", error));
        self.meta
            .insert(META_HEIGHT_KEY, &StorageCodec::encode_height(next_height))
            .unwrap_or_else(|error| panic!("failed to write height: {}", error));
        self.flush();
    }

    fn flush(&self) {
        self.db
            .flush()
            .unwrap_or_else(|error| panic!("failed to flush sled db: {}", error));
    }

    fn load_accounts(&self) -> BTreeMap<Address, Account> {
        let mut accounts = BTreeMap::new();
        for entry in self.accounts.iter() {
            let (key, value) =
                entry.unwrap_or_else(|error| panic!("failed to iterate accounts tree: {}", error));
            let address = StorageCodec::decode_address(key.as_ref())
                .unwrap_or_else(|error| panic!("failed to decode account key: {}", error));
            let account = StorageCodec::decode_account(value.as_ref())
                .unwrap_or_else(|error| panic!("failed to decode account value: {}", error));
            accounts.insert(address, account);
        }
        accounts
    }

    fn load_block(&self, height: Height) -> Option<Block> {
        self.blocks
            .get(StorageCodec::encode_height(height))
            .unwrap_or_else(|error| panic!("failed to read block {}: {}", height, error))
            .map(|bytes| {
                StorageCodec::decode_block(bytes.as_ref())
                    .unwrap_or_else(|error| panic!("failed to decode block {}: {}", height, error))
            })
    }

    fn load_contract_storage(&self) -> BTreeMap<(Address, Hash), Hash> {
        let mut entries = BTreeMap::new();
        for entry in self.contract_storage.iter() {
            let (key, value) = entry.unwrap_or_else(|error| {
                panic!("failed to iterate contract storage tree: {}", error)
            });
            let decoded_key = StorageCodec::decode_contract_storage_key(key.as_ref())
                .unwrap_or_else(|error| panic!("failed to decode contract storage key: {}", error));
            let decoded_value = StorageCodec::decode_hash(value.as_ref()).unwrap_or_else(|error| {
                panic!("failed to decode contract storage value: {}", error)
            });
            entries.insert(decoded_key, decoded_value);
        }
        entries
    }

    fn load_height(&self) -> Height {
        self.meta
            .get(META_HEIGHT_KEY)
            .unwrap_or_else(|error| panic!("failed to read height: {}", error))
            .map(|bytes| {
                StorageCodec::decode_height(bytes.as_ref())
                    .unwrap_or_else(|error| panic!("failed to decode height: {}", error))
            })
            .unwrap_or(0)
    }

    fn load_receipts(&self, height: Height) -> Vec<TransactionReceipt> {
        self.receipts
            .get(StorageCodec::encode_height(height))
            .unwrap_or_else(|error| panic!("failed to read receipts {}: {}", height, error))
            .map(|bytes| {
                StorageCodec::decode_receipts(bytes.as_ref()).unwrap_or_else(|error| {
                    panic!("failed to decode receipts {}: {}", height, error)
                })
            })
            .unwrap_or_default()
    }

    fn load_state_root(&self) -> Hash {
        self.meta
            .get(META_STATE_ROOT_KEY)
            .unwrap_or_else(|error| panic!("failed to read state root: {}", error))
            .map(|bytes| {
                StorageCodec::decode_hash(bytes.as_ref())
                    .unwrap_or_else(|error| panic!("failed to decode state root: {}", error))
            })
            .unwrap_or([0u8; 32])
    }

    fn mutate(&mut self, mutation: StorageMutation) {
        match mutation {
            StorageMutation::UpdateAccount(address, account) => {
                let accounts_key = StorageCodec::encode_account_key(&address);
                let accounts_value = StorageCodec::encode_account(&account)
                    .unwrap_or_else(|error| panic!("failed to encode account: {}", error));
                self.accounts
                    .insert(accounts_key, accounts_value)
                    .unwrap_or_else(|error| panic!("failed to write account: {}", error));
                let next_state_root = self.recompute_state_root();
                self.meta
                    .insert(
                        META_STATE_ROOT_KEY,
                        StorageCodec::encode_hash(&next_state_root),
                    )
                    .unwrap_or_else(|error| panic!("failed to write state root: {}", error));
                self.flush();
            }
            StorageMutation::UpdateContractStorage {
                address,
                slot,
                value,
            } => {
                let contract_key = StorageCodec::encode_contract_storage_key(&address, &slot);
                let contract_value = StorageCodec::encode_hash(&value);
                self.contract_storage
                    .insert(contract_key, contract_value)
                    .unwrap_or_else(|error| panic!("failed to write contract storage: {}", error));
                let next_state_root = self.recompute_state_root();
                self.meta
                    .insert(
                        META_STATE_ROOT_KEY,
                        StorageCodec::encode_hash(&next_state_root),
                    )
                    .unwrap_or_else(|error| panic!("failed to write state root: {}", error));
                self.flush();
            }
            StorageMutation::IncrementHeight => {
                let next_height = self.load_height() + 1;
                self.meta
                    .insert(META_HEIGHT_KEY, &StorageCodec::encode_height(next_height))
                    .unwrap_or_else(|error| panic!("failed to write height: {}", error));
                self.flush();
            }
            StorageMutation::StoreBlock(block) => {
                let height = block.height;
                let encoded = StorageCodec::encode_block(&block)
                    .unwrap_or_else(|error| panic!("failed to encode block {}: {}", height, error));
                self.blocks
                    .insert(StorageCodec::encode_height(height), encoded)
                    .unwrap_or_else(|error| panic!("failed to store block {}: {}", height, error));
                self.flush();
            }
            StorageMutation::StoreReceipts(height, receipts) => {
                let encoded = StorageCodec::encode_receipts(&receipts).unwrap_or_else(|error| {
                    panic!("failed to encode receipts {}: {}", height, error)
                });
                self.receipts
                    .insert(StorageCodec::encode_height(height), encoded)
                    .unwrap_or_else(|error| {
                        panic!("failed to store receipts {}: {}", height, error)
                    });
                self.flush();
            }
        }
    }

    fn new(db_path: &str) -> Result<Self, String> {
        fs::create_dir_all(Path::new(db_path))
            .map_err(|error| format!("failed to create db directory {}: {error}", db_path))?;
        let db = open(db_path)
            .map_err(|error| format!("failed to open sled db {}: {error}", db_path))?;
        let accounts = db
            .open_tree("accounts")
            .map_err(|error| format!("failed to open accounts tree: {error}"))?;
        let blocks = db
            .open_tree("blocks")
            .map_err(|error| format!("failed to open blocks tree: {error}"))?;
        let contract_storage = db
            .open_tree("contract_storage")
            .map_err(|error| format!("failed to open contract_storage tree: {error}"))?;
        let meta = db
            .open_tree("meta")
            .map_err(|error| format!("failed to open meta tree: {error}"))?;
        let receipts = db
            .open_tree("receipts")
            .map_err(|error| format!("failed to open receipts tree: {error}"))?;

        Ok(Self {
            accounts,
            blocks,
            contract_storage,
            db,
            meta,
            receipts,
        })
    }

    fn query(&self, query: StorageQuery) -> StorageQueryResult {
        match query {
            StorageQuery::GetAccount(address) => StorageQueryResult::Account(
                self.accounts
                    .get(StorageCodec::encode_account_key(&address))
                    .unwrap_or_else(|error| panic!("failed to read account: {}", error))
                    .map(|bytes| {
                        StorageCodec::decode_account(bytes.as_ref())
                            .unwrap_or_else(|error| panic!("failed to decode account: {}", error))
                    }),
            ),
            StorageQuery::GetContractStorage { address, slot } => {
                StorageQueryResult::ContractStorage(
                    self.contract_storage
                        .get(StorageCodec::encode_contract_storage_key(&address, &slot))
                        .unwrap_or_else(|error| {
                            panic!("failed to read contract storage: {}", error)
                        })
                        .map(|bytes| {
                            StorageCodec::decode_hash(bytes.as_ref()).unwrap_or_else(|error| {
                                panic!("failed to decode contract storage value: {}", error)
                            })
                        }),
                )
            }
            StorageQuery::GetAllAccounts => StorageQueryResult::Accounts(self.load_accounts()),
            StorageQuery::GetAllContractStorage => {
                StorageQueryResult::ContractStorageEntries(self.load_contract_storage())
            }
            StorageQuery::GetHeight => StorageQueryResult::Height(self.load_height()),
            StorageQuery::GetStateRoot => StorageQueryResult::StateRoot(self.load_state_root()),
            StorageQuery::GetBlock(height) => StorageQueryResult::Block(self.load_block(height)),
            StorageQuery::GetReceipts(height) => {
                StorageQueryResult::Receipts(self.load_receipts(height))
            }
        }
    }

    fn recompute_state_root(&self) -> Hash {
        let accounts = self.load_accounts();
        let contract_storage = self.load_contract_storage();
        compute_state_root_with_contract_storage(&accounts, &contract_storage)
    }
}
