// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_db_path;
use crate::common::test_config::create_test_db_path;
use etheram_core::storage::Storage;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::execution::transaction_receipt::TransactionReceipt;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use etheram_node_process::infra::storage::sled_storage::SledStorage;

#[test]
fn apply_synced_blocks_two_blocks_persists_blocks_and_height_across_reopen() {
    // Arrange
    let db_path = create_test_db_path("sled_sync_blocks");
    let storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let mut block_0 = Block::new(
        0,
        1,
        vec![Transaction::new(
            [1u8; 20],
            [2u8; 20],
            55,
            30_000,
            3,
            0,
            vec![1, 2, 3],
        )],
        [1u8; 32],
        40_000,
    );
    block_0.post_state_root = [3u8; 32];
    block_0.receipts_root = [4u8; 32];
    let mut block_1 = Block::new(
        1,
        1,
        vec![Transaction::new(
            [5u8; 20],
            [6u8; 20],
            77,
            35_000,
            4,
            1,
            vec![4, 5],
        )],
        block_0.post_state_root,
        50_000,
    );
    block_1.post_state_root = [7u8; 32];
    block_1.receipts_root = [8u8; 32];

    // Act
    storage.apply_synced_blocks(&[block_0.clone(), block_1.clone()]);
    drop(storage);
    let reopened =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to reopen storage");

    // Assert
    match reopened.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => assert_eq!(height, 2),
        _ => panic!("unexpected query result for height"),
    }
    match reopened.query(StorageQuery::GetBlock(0)) {
        StorageQueryResult::Block(Some(block)) => assert_eq!(block, block_0),
        _ => panic!("unexpected query result for block 0"),
    }
    match reopened.query(StorageQuery::GetBlock(1)) {
        StorageQueryResult::Block(Some(block)) => assert_eq!(block, block_1),
        _ => panic!("unexpected query result for block 1"),
    }

    cleanup_test_db_path(&db_path);
}

#[test]
fn mutate_update_account_persists_account_and_state_root_across_reopen() {
    // Arrange
    let db_path = create_test_db_path("sled_account_state_root");
    let mut storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let address: Address = [7u8; 20];
    let account = Account::new(123);

    // Act
    storage.mutate(StorageMutation::UpdateAccount(address, account.clone()));
    let state_root_before = match storage.query(StorageQuery::GetStateRoot) {
        StorageQueryResult::StateRoot(state_root) => state_root,
        _ => panic!("unexpected query result for state root"),
    };
    drop(storage);
    let reopened =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to reopen storage");

    // Assert
    match reopened.query(StorageQuery::GetAccount(address)) {
        StorageQueryResult::Account(Some(restored)) => assert_eq!(restored, account),
        _ => panic!("unexpected query result for account"),
    }
    match reopened.query(StorageQuery::GetStateRoot) {
        StorageQueryResult::StateRoot(state_root) => assert_eq!(state_root, state_root_before),
        _ => panic!("unexpected query result for state root"),
    }

    cleanup_test_db_path(&db_path);
}

#[test]
fn mutate_update_contract_storage_persists_value_across_reopen() {
    // Arrange
    let db_path = create_test_db_path("sled_contract_storage");
    let mut storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let address: Address = [8u8; 20];
    let slot = [3u8; 32];
    let value = [4u8; 32];

    // Act
    storage.mutate(StorageMutation::UpdateContractStorage {
        address,
        slot,
        value,
    });
    drop(storage);
    let reopened =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to reopen storage");

    // Assert
    match reopened.query(StorageQuery::GetContractStorage { address, slot }) {
        StorageQueryResult::ContractStorage(Some(restored)) => assert_eq!(restored, value),
        _ => panic!("unexpected query result for contract storage"),
    }

    cleanup_test_db_path(&db_path);
}

#[test]
fn mutate_store_receipts_persists_receipts_across_reopen() {
    // Arrange
    let db_path = create_test_db_path("sled_receipts");
    let mut storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let receipts = vec![
        TransactionReceipt {
            status: TransactionStatus::Success,
            gas_used: 21_000,
            cumulative_gas_used: 21_000,
        },
        TransactionReceipt {
            status: TransactionStatus::OutOfGas,
            gas_used: 50_000,
            cumulative_gas_used: 71_000,
        },
        TransactionReceipt {
            status: TransactionStatus::Reverted,
            gas_used: 13_000,
            cumulative_gas_used: 84_000,
        },
        TransactionReceipt {
            status: TransactionStatus::InvalidOpcode,
            gas_used: 1_100,
            cumulative_gas_used: 85_100,
        },
    ];

    // Act
    storage.mutate(StorageMutation::StoreReceipts(0, receipts.clone()));
    drop(storage);
    let reopened =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to reopen storage");

    // Assert
    match reopened.query(StorageQuery::GetReceipts(0)) {
        StorageQueryResult::Receipts(restored) => assert_eq!(restored, receipts),
        _ => panic!("unexpected query result for receipts"),
    }

    cleanup_test_db_path(&db_path);
}
