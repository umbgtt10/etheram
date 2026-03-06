// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::transaction::Transaction;
use etheram_node_process::infra::sync::sync_import::decode_and_validate_blocks;
use etheram_node_process::infra::transport::grpc_transport::wire_ibft_message::serialize_block;

#[test]
fn decode_and_validate_blocks_matching_start_and_contiguous_heights_returns_blocks() {
    // Arrange
    let block_3 = Block::empty(3, 1, [1u8; 32]);
    let block_4 = Block::empty(4, 1, [2u8; 32]);
    let payload_3 = serialize_block(&block_3).expect("failed to serialize block 3");
    let payload_4 = serialize_block(&block_4).expect("failed to serialize block 4");

    // Act
    let decoded = decode_and_validate_blocks(3, 3, &[payload_3, payload_4], None);

    // Assert
    assert!(decoded.is_some());
    let blocks = decoded.expect("expected decoded blocks");
    assert_eq!(blocks, vec![block_3, block_4]);
}

#[test]
fn decode_and_validate_blocks_start_height_not_equal_local_height_returns_none() {
    // Arrange
    let block_4 = Block::empty(4, 1, [3u8; 32]);
    let payload_4 = serialize_block(&block_4).expect("failed to serialize block 4");

    // Act
    let decoded = decode_and_validate_blocks(3, 4, &[payload_4], None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_non_contiguous_height_returns_none() {
    // Arrange
    let block_3 = Block::empty(3, 1, [4u8; 32]);
    let block_5 = Block::empty(5, 1, [5u8; 32]);
    let payload_3 = serialize_block(&block_3).expect("failed to serialize block 3");
    let payload_5 = serialize_block(&block_5).expect("failed to serialize block 5");

    // Act
    let decoded = decode_and_validate_blocks(3, 3, &[payload_3, payload_5], None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_invalid_payload_returns_none() {
    // Arrange
    let invalid_payload = vec![1u8, 2u8, 3u8];

    // Act
    let decoded = decode_and_validate_blocks(0, 0, &[invalid_payload], None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_empty_payloads_matching_start_returns_empty_blocks() {
    // Arrange
    let payloads: Vec<Vec<u8>> = Vec::new();

    // Act
    let decoded = decode_and_validate_blocks(5, 5, &payloads, None);

    // Assert
    assert!(decoded.is_some());
    let blocks = decoded.expect("expected decoded empty block range");
    assert!(blocks.is_empty());
}

#[test]
fn decode_and_validate_blocks_empty_payloads_start_height_mismatch_returns_none() {
    // Arrange
    let payloads: Vec<Vec<u8>> = Vec::new();

    // Act
    let decoded = decode_and_validate_blocks(5, 6, &payloads, None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_first_block_not_matching_start_height_returns_none() {
    // Arrange
    let block_6 = Block::empty(6, 1, [6u8; 32]);
    let payload_6 = serialize_block(&block_6).expect("failed to serialize block 6");

    // Act
    let decoded = decode_and_validate_blocks(5, 5, &[payload_6], None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_excessive_block_gas_limit_returns_none() {
    // Arrange
    let block = Block {
        height: 0,
        proposer: 1,
        transactions: Vec::new(),
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
        gas_limit: BLOCK_GAS_LIMIT + 1,
    };
    let payload = serialize_block(&block).expect("failed to serialize oversized gas block");

    // Act
    let decoded = decode_and_validate_blocks(0, 0, &[payload], None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_tx_gas_sum_over_block_limit_returns_none() {
    // Arrange
    let tx = Transaction::transfer([1u8; 20], [2u8; 20], 1, BLOCK_GAS_LIMIT, 1, 0);
    let block = Block {
        height: 0,
        proposer: 1,
        transactions: vec![tx],
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
        gas_limit: BLOCK_GAS_LIMIT - 1,
    };
    let payload = serialize_block(&block).expect("failed to serialize over-limit tx-gas block");

    // Act
    let decoded = decode_and_validate_blocks(0, 0, &[payload], None);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_parent_post_state_root_mismatch_returns_none() {
    // Arrange
    let block = Block {
        height: 3,
        proposer: 1,
        transactions: Vec::new(),
        state_root: [8u8; 32],
        post_state_root: [9u8; 32],
        receipts_root: [0u8; 32],
        gas_limit: BLOCK_GAS_LIMIT,
    };
    let payload = serialize_block(&block).expect("failed to serialize block");

    // Act
    let decoded = decode_and_validate_blocks(3, 3, &[payload], Some([7u8; 32]));

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_parent_post_state_root_match_returns_blocks() {
    // Arrange
    let block = Block {
        height: 3,
        proposer: 1,
        transactions: Vec::new(),
        state_root: [7u8; 32],
        post_state_root: [9u8; 32],
        receipts_root: [0u8; 32],
        gas_limit: BLOCK_GAS_LIMIT,
    };
    let payload = serialize_block(&block).expect("failed to serialize block");

    // Act
    let decoded = decode_and_validate_blocks(3, 3, &[payload], Some([7u8; 32]));

    // Assert
    assert!(decoded.is_some());
}
