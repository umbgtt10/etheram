// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_db_path;
use crate::common::test_config::create_test_db_path;
use etheram_core::storage::Storage;
use etheram_node::common_types::block::Block;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node_process::etheram_node::NodeRuntime;
use etheram_node_process::infra::storage::sled_storage::SledStorage;
use std::collections::BTreeMap;

#[test]
fn new_existing_db_restores_height_and_last_hash_across_restart() {
    // Arrange
    let db_path = create_test_db_path("node_runtime_restart");
    let mut storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let block = Block::empty(0, 1, [11u8; 32]);
    let expected_last_hash = block.compute_hash();
    storage.mutate(StorageMutation::StoreBlock(block));
    storage.mutate(StorageMutation::IncrementHeight);
    drop(storage);
    let mut peer_addresses = BTreeMap::new();
    peer_addresses.insert(1, "127.0.0.1:0".to_string());

    // Act
    let runtime_1 = NodeRuntime::new(
        1,
        "127.0.0.1:0",
        &peer_addresses,
        &[1],
        db_path.to_string_lossy().as_ref(),
    )
    .expect("failed to build first runtime");
    let height_1 = runtime_1.current_height();
    let hash_1 = runtime_1.last_block_hash();
    drop(runtime_1);
    let runtime_2 = NodeRuntime::new(
        1,
        "127.0.0.1:0",
        &peer_addresses,
        &[1],
        db_path.to_string_lossy().as_ref(),
    )
    .expect("failed to build second runtime");

    // Assert
    assert_eq!(height_1, 1);
    assert_eq!(hash_1, expected_last_hash);
    assert_eq!(runtime_2.current_height(), 1);
    assert_eq!(runtime_2.last_block_hash(), expected_last_hash);

    cleanup_test_db_path(&db_path);
}
