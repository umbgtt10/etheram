// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::injected_storage::InjectedStorage;
use etheram_core::storage::Storage;
use etheram_node::common_types::block::Block;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;

#[test]
fn apply_synced_blocks_two_blocks_stores_blocks_and_increments_height() {
    // Arrange
    let storage = InjectedStorage::new().expect("failed to build injected storage");
    let handle = storage.handle();
    let block_0 = Block::empty(0, 1, [1u8; 32]);
    let block_1 = Block::empty(1, 1, [2u8; 32]);

    // Act
    handle.apply_synced_blocks(&[block_0.clone(), block_1.clone()]);

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
