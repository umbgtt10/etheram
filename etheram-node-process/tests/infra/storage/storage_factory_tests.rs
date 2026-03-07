// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_db_path;
use crate::common::test_config::create_test_db_path;
use etheram_core::storage::Storage;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::storage_adapter::StorageAdapter;
use etheram_node::common_types::types::Address;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use etheram_node_process::infra::storage::storage_factory::build_storage;

#[test]
fn build_storage_clone_and_box_share_state_in_both_directions() {
    // Arrange
    let db_path = create_test_db_path("storage_factory_share_state");
    let storage =
        build_storage(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let sync_storage = storage.clone();
    let mut adapter: Box<dyn StorageAdapter<Key = Address, Value = Account>> = Box::new(storage);
    let block_0 = Block::empty(0, 1, [9u8; 32]);

    // Act
    sync_storage.apply_synced_blocks(std::slice::from_ref(&block_0));

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
    match sync_storage.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => {
            assert_eq!(height, 2);
        }
        _ => panic!("unexpected query result for height after adapter mutate"),
    }

    cleanup_test_db_path(&db_path);
}
