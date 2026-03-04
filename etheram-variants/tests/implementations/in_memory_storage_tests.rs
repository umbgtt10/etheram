// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::types::Address;
use etheram::common_types::types::Hash;
use etheram::state::storage::storage_mutation::StorageMutation;
use etheram::state::storage::storage_query::StorageQuery;
use etheram::state::storage::storage_query_result::StorageQueryResult;
use etheram_core::storage::Storage;
use etheram_etheram_variants::implementations::in_memory_storage::InMemoryStorage;

#[test]
fn with_genesis_account_known_address_returns_balance() {
    // Arrange
    let addr: Address = [1u8; 20];

    // Act
    let storage = InMemoryStorage::new().with_genesis_account(addr, 500);

    // Assert
    let result = storage.query(StorageQuery::GetAccount(addr));
    let StorageQueryResult::Account(Some(account)) = result else {
        panic!("expected account");
    };
    assert_eq!(account.balance, 500);
}

#[test]
fn query_get_account_unknown_address_returns_none() {
    // Arrange
    let storage = InMemoryStorage::new();
    let addr: Address = [2u8; 20];

    // Act
    let result = storage.query(StorageQuery::GetAccount(addr));

    // Assert
    assert!(matches!(result, StorageQueryResult::Account(None)));
}

#[test]
fn mutate_update_account_query_returns_updated_account() {
    // Arrange
    let mut storage = InMemoryStorage::new();
    let addr: Address = [3u8; 20];
    let account = Account::new(1000);

    // Act
    storage.mutate(StorageMutation::UpdateAccount(addr, account.clone()));

    // Assert
    let result = storage.query(StorageQuery::GetAccount(addr));
    let StorageQueryResult::Account(Some(stored)) = result else {
        panic!("expected account");
    };
    assert_eq!(stored.balance, 1000);
}

#[test]
fn mutate_increment_height_query_returns_incremented_value() {
    // Arrange
    let mut storage = InMemoryStorage::new();

    // Act
    storage.mutate(StorageMutation::IncrementHeight);

    // Assert
    let result = storage.query(StorageQuery::GetHeight);
    assert!(matches!(result, StorageQueryResult::Height(1)));
}

#[test]
fn mutate_store_block_query_returns_stored_block() {
    // Arrange
    let mut storage = InMemoryStorage::new();
    let block = Block::new(0, 0, vec![], [0u8; 32]);

    // Act
    storage.mutate(StorageMutation::StoreBlock(block.clone()));

    // Assert
    let result = storage.query(StorageQuery::GetBlock(0));
    let StorageQueryResult::Block(Some(stored)) = result else {
        panic!("expected block");
    };
    assert_eq!(stored.height, 0);
}

#[test]
fn query_get_block_out_of_range_returns_none() {
    // Arrange
    let storage = InMemoryStorage::new();

    // Act
    let result = storage.query(StorageQuery::GetBlock(99));

    // Assert
    assert!(matches!(result, StorageQueryResult::Block(None)));
}

#[test]
fn query_get_all_accounts_with_genesis_accounts_returns_snapshot() {
    // Arrange
    let addr_a: Address = [4u8; 20];
    let addr_b: Address = [5u8; 20];
    let storage = InMemoryStorage::new()
        .with_genesis_account(addr_a, 10)
        .with_genesis_account(addr_b, 20);

    // Act
    let result = storage.query(StorageQuery::GetAllAccounts);

    // Assert
    let StorageQueryResult::Accounts(accounts) = result else {
        panic!("expected accounts snapshot");
    };
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts.get(&addr_a).map(|a| a.balance), Some(10));
    assert_eq!(accounts.get(&addr_b).map(|a| a.balance), Some(20));
}

#[test]
fn query_get_all_contract_storage_default_storage_returns_empty() {
    // Arrange
    let storage = InMemoryStorage::new();

    // Act
    let result = storage.query(StorageQuery::GetAllContractStorage);

    // Assert
    let StorageQueryResult::ContractStorageEntries(entries) = result else {
        panic!("expected contract storage snapshot");
    };
    assert!(entries.is_empty());
}

#[test]
fn mutate_update_contract_storage_query_contract_storage_returns_stored_value() {
    // Arrange
    let mut storage = InMemoryStorage::new();
    let address: Address = [6u8; 20];
    let slot: Hash = [7u8; 32];
    let value: Hash = [8u8; 32];

    // Act
    storage.mutate(StorageMutation::UpdateContractStorage {
        address,
        slot,
        value,
    });
    let result = storage.query(StorageQuery::GetContractStorage { address, slot });

    // Assert
    assert!(matches!(result, StorageQueryResult::ContractStorage(Some(v)) if v == value));
}

#[test]
fn mutate_update_contract_storage_query_state_root_returns_updated_root() {
    // Arrange
    let mut storage = InMemoryStorage::new();
    let address: Address = [9u8; 20];
    let slot: Hash = [10u8; 32];
    let value: Hash = [11u8; 32];
    let initial_root = match storage.query(StorageQuery::GetStateRoot) {
        StorageQueryResult::StateRoot(root) => root,
        _ => panic!("expected state root"),
    };

    // Act
    storage.mutate(StorageMutation::UpdateContractStorage {
        address,
        slot,
        value,
    });
    let updated_root = match storage.query(StorageQuery::GetStateRoot) {
        StorageQueryResult::StateRoot(root) => root,
        _ => panic!("expected state root"),
    };

    // Assert
    assert_ne!(initial_root, updated_root);
}
