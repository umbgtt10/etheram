// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::in_memory_storage::InMemoryStorage;
use crate::infra::sync::sync_import::decode_and_validate_blocks;
use crate::infra::sync::sync_state::SyncState;
use crate::infra::transport::grpc_transport::wire_ibft_message::serialize_block;
use etheram_core::storage::Storage;
use etheram_node::common_types::block::Block;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;

#[test]
fn status_observation_then_request_planning_returns_expected_peer_and_range() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 14);
    state.observe_status(3, 16);
    let local_height = 10;

    // Act
    let planned = state.next_request(local_height, 64);

    // Assert
    assert_eq!(planned, Some((3, 10, 64)));
}

#[test]
fn decoded_blocks_import_then_storage_height_and_blocks_advance() {
    // Arrange
    let storage = InMemoryStorage::new().expect("failed to build storage");
    let block_0 = Block::empty(0, 1, [1u8; 32]);
    let block_1 = Block::empty(1, 1, [2u8; 32]);
    let payload_0 = serialize_block(&block_0).expect("failed to serialize block 0");
    let payload_1 = serialize_block(&block_1).expect("failed to serialize block 1");

    // Act
    let decoded = decode_and_validate_blocks(0, 0, &[payload_0, payload_1], None);
    let blocks = decoded.expect("expected valid decoded blocks");
    storage.apply_synced_blocks(&blocks);

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
fn malformed_or_empty_lagging_response_then_failover_selects_next_peer() {
    // Arrange
    let mut state = SyncState::new();
    state.observe_status(2, 20);
    state.observe_status(3, 20);
    let local_height = 10;
    let first = state
        .next_request(local_height, 64)
        .expect("expected first request");

    // Act
    let malformed_payload = vec![1u8, 2u8, 3u8];
    let malformed_decoded =
        decode_and_validate_blocks(local_height, first.1, &[malformed_payload], None);
    let malformed_failed = if malformed_decoded.is_none() {
        state.fail_in_flight_request(first.0, first.1)
    } else {
        false
    };
    let second = state
        .next_request(local_height, 64)
        .expect("expected failover request");
    let empty_payloads: Vec<Vec<u8>> = Vec::new();
    let empty_decoded = decode_and_validate_blocks(local_height, second.1, &empty_payloads, None);
    let lag_distance = state.lag_distance(local_height);
    let empty_while_lagging = empty_payloads.is_empty() && lag_distance.is_some();
    let empty_failed = if empty_decoded.is_some() && empty_while_lagging {
        state.fail_in_flight_request(second.0, second.1)
    } else {
        false
    };

    // Assert
    assert!(malformed_failed);
    assert!(empty_failed);
    assert_ne!(second.0, first.0);
}
