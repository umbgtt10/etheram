// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::storage::Storage;
use etheram_node::common_types::block::Block;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use etheram_node_process::infra::storage::in_memory_storage::InMemoryStorage;
use etheram_node_process::infra::sync::sync_import::decode_and_validate_blocks;
use etheram_node_process::infra::sync::sync_state::SyncState;
use etheram_node_process::infra::transport::grpc_transport::wire_ibft_message::serialize_block;

#[test]
fn partition_and_heal_lag_recovery_selects_new_request_and_imports_after_heal() {
    // Arrange
    let mut state = SyncState::new();
    let storage = InMemoryStorage::new().expect("failed to build storage");
    state.observe_status(2, 3);
    state.observe_status(3, 3);
    let first = state.next_request(0, 64).expect("expected initial request");
    let first_failed = state.fail_in_flight_request(first.0, first.1);
    let failover = state
        .next_request(0, 64)
        .expect("expected failover request");
    let block_0 = Block::empty(0, 1, [1u8; 32]);
    let block_1 = Block::empty(1, 1, [2u8; 32]);
    let block_2 = Block::empty(2, 1, [3u8; 32]);
    let payload_0 = serialize_block(&block_0).expect("failed to serialize block 0");
    let payload_1 = serialize_block(&block_1).expect("failed to serialize block 1");
    let payload_2 = serialize_block(&block_2).expect("failed to serialize block 2");

    // Act
    let decoded =
        decode_and_validate_blocks(0, failover.1, &[payload_0, payload_1, payload_2], None)
            .expect("expected decoded blocks after heal");
    storage.apply_synced_blocks(&decoded);
    let completed = state.complete_in_flight_request(failover.0, failover.1);

    // Assert
    assert!(first_failed);
    assert!(completed);
    match storage.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => assert_eq!(height, 3),
        _ => panic!("unexpected query result for height"),
    }
}

#[test]
fn long_partition_multi_batch_sync_import_catches_up_fully() {
    // Arrange
    let mut state = SyncState::new();
    let storage = InMemoryStorage::new().expect("failed to build storage");
    state.observe_status(2, 6);

    let batch_1_block_0 = Block::empty(0, 1, [10u8; 32]);
    let batch_1_block_1 = Block::empty(1, 1, [11u8; 32]);
    let batch_1_block_2 = Block::empty(2, 1, [12u8; 32]);

    let batch_2_block_3 = Block::empty(3, 1, [13u8; 32]);
    let batch_2_block_4 = Block::empty(4, 1, [14u8; 32]);
    let batch_2_block_5 = Block::empty(5, 1, [15u8; 32]);

    // Act
    let first = state.next_request(0, 3).expect("expected first request");
    let first_decoded = decode_and_validate_blocks(
        0,
        first.1,
        &[
            serialize_block(&batch_1_block_0).expect("serialize 0"),
            serialize_block(&batch_1_block_1).expect("serialize 1"),
            serialize_block(&batch_1_block_2).expect("serialize 2"),
        ],
        None,
    )
    .expect("expected first decoded batch");
    storage.apply_synced_blocks(&first_decoded);
    let first_completed = state.complete_in_flight_request(first.0, first.1);

    let second = state.next_request(3, 3).expect("expected second request");
    let second_decoded = decode_and_validate_blocks(
        3,
        second.1,
        &[
            serialize_block(&batch_2_block_3).expect("serialize 3"),
            serialize_block(&batch_2_block_4).expect("serialize 4"),
            serialize_block(&batch_2_block_5).expect("serialize 5"),
        ],
        None,
    )
    .expect("expected second decoded batch");
    storage.apply_synced_blocks(&second_decoded);
    let second_completed = state.complete_in_flight_request(second.0, second.1);

    // Assert
    assert!(first_completed);
    assert!(second_completed);
    match storage.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => assert_eq!(height, 6),
        _ => panic!("unexpected query result for height"),
    }
}

#[test]
fn invalid_range_response_is_rejected_and_failover_is_planned() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 4);
    state.observe_status(3, 4);
    let first = state.next_request(0, 64).expect("expected first request");
    let invalid_payload = vec![9u8, 8u8, 7u8];

    // Act
    let decoded = decode_and_validate_blocks(0, first.1, &[invalid_payload], None);
    let failed = if decoded.is_none() {
        state.fail_in_flight_request(first.0, first.1)
    } else {
        false
    };
    let failover = state.next_request(0, 64);

    // Assert
    assert!(failed);
    assert!(failover.is_some());
    let planned = failover.expect("expected failover request");
    assert_ne!(planned.0, first.0);
}

#[test]
fn active_sync_peer_offline_mid_sync_switches_peer_and_completes() {
    // Arrange
    let mut state = SyncState::new();
    let storage = InMemoryStorage::new().expect("failed to build storage");
    state.observe_status(2, 2);
    state.observe_status(3, 2);
    let first = state.next_request(0, 64).expect("expected first request");
    let first_failed = state.fail_in_flight_request(first.0, first.1);
    let second = state
        .next_request(0, 64)
        .expect("expected fallback request");
    let block_0 = Block::empty(0, 1, [21u8; 32]);
    let block_1 = Block::empty(1, 1, [22u8; 32]);

    // Act
    let decoded = decode_and_validate_blocks(
        0,
        second.1,
        &[
            serialize_block(&block_0).expect("serialize block 0"),
            serialize_block(&block_1).expect("serialize block 1"),
        ],
        None,
    )
    .expect("expected decoded blocks");
    storage.apply_synced_blocks(&decoded);
    let completed = state.complete_in_flight_request(second.0, second.1);

    // Assert
    assert!(first_failed);
    assert!(completed);
    assert_ne!(second.0, first.0);
    match storage.query(StorageQuery::GetHeight) {
        StorageQueryResult::Height(height) => assert_eq!(height, 2),
        _ => panic!("unexpected query result for height"),
    }
}

#[test]
fn no_op_sync_at_tip_plans_no_request() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 5);
    state.observe_status(3, 5);

    // Act
    let planned = state.next_request(5, 64);

    // Assert
    assert!(planned.is_none());
}
