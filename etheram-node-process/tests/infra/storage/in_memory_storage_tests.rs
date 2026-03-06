// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::storage::Storage;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::storage_adapter::StorageAdapter;
use etheram_node::common_types::types::Address;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use etheram_node_process::infra::storage::in_memory_storage::InMemoryStorage;

#[test]
fn apply_synced_blocks_two_blocks_stores_blocks_and_increments_height() {
    // Arrange
    let storage = InMemoryStorage::new().expect("failed to build in-memory storage");
    let block_0 = Block::empty(0, 1, [1u8; 32]);
    let block_1 = Block::empty(1, 1, [2u8; 32]);

    // Act
    storage.apply_synced_blocks(&[block_0.clone(), block_1.clone()]);

    // Assert
    match storage.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => {
            assert_eq!(height, 2);
        }
        _ => panic!("unexpected query result for height"),
    }

    match storage.query(StorageQuery::GetBlock(0)) {
        StorageQueryResult::Block(Some(block)) => {
            assert_eq!(block, block_0);
        }
        _ => panic!("unexpected query result for block 0"),
    }

    match storage.query(StorageQuery::GetBlock(1)) {
        StorageQueryResult::Block(Some(block)) => {
            assert_eq!(block, block_1);
        }
        _ => panic!("unexpected query result for block 1"),
    }
}

#[test]
fn apply_synced_blocks_on_clone_updates_boxed_adapter_view() {
    // Arrange
    let storage = InMemoryStorage::new().expect("failed to build in-memory storage");
    let mut adapter: Box<dyn StorageAdapter<Key = Address, Value = Account>> =
        Box::new(storage.clone());
    let block_0 = Block::empty(0, 1, [3u8; 32]);

    // Act
    storage.apply_synced_blocks(std::slice::from_ref(&block_0));

    // Assert
    match adapter.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => {
            assert_eq!(height, 1);
        }
        _ => panic!("unexpected query result for height"),
    }

    match adapter.query(StorageQuery::GetBlock(0)) {
        StorageQueryResult::Block(Some(block)) => {
            assert_eq!(block, block_0);
        }
        _ => panic!("unexpected query result for block 0"),
    }

    adapter.mutate(StorageMutation::IncrementHeight);
    match storage.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => {
            assert_eq!(height, 2);
        }
        _ => panic!("unexpected query result for height after adapter mutate"),
    }
}
