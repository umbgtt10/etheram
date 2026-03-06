// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::sync::sync_import::decode_and_validate_blocks;
use crate::infra::transport::grpc_transport::wire_ibft_message::serialize_block;
use etheram_node::common_types::block::Block;

#[test]
fn decode_and_validate_blocks_matching_start_and_contiguous_heights_returns_blocks() {
    // Arrange
    let block_3 = Block::empty(3, 1, [1u8; 32]);
    let block_4 = Block::empty(4, 1, [2u8; 32]);
    let payload_3 = serialize_block(&block_3).expect("failed to serialize block 3");
    let payload_4 = serialize_block(&block_4).expect("failed to serialize block 4");

    // Act
    let decoded = decode_and_validate_blocks(3, 3, &[payload_3, payload_4]);

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
    let decoded = decode_and_validate_blocks(3, 4, &[payload_4]);

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
    let decoded = decode_and_validate_blocks(3, 3, &[payload_3, payload_5]);

    // Assert
    assert!(decoded.is_none());
}

#[test]
fn decode_and_validate_blocks_invalid_payload_returns_none() {
    // Arrange
    let invalid_payload = vec![1u8, 2u8, 3u8];

    // Act
    let decoded = decode_and_validate_blocks(0, 0, &[invalid_payload]);

    // Assert
    assert!(decoded.is_none());
}
