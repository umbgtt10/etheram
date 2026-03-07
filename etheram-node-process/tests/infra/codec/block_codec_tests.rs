// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node_process::infra::codec::block_codec::BlockCodec;

#[test]
fn serialize_non_empty_block_round_trips_exactly() {
    // Arrange
    let transactions = vec![
        Transaction::new([1u8; 20], [2u8; 20], 11, 21_000, 7, 0, vec![1, 2, 3]),
        Transaction::new([3u8; 20], [4u8; 20], 29, 45_000, 9, 1, vec![4, 5, 6, 7]),
    ];
    let mut block = Block::new(4, 2, transactions, [8u8; 32], 90_000);
    block.post_state_root = [9u8; 32];
    block.receipts_root = [10u8; 32];
    let encoded = BlockCodec::serialize(&block).expect("failed to serialize block");

    // Act
    let decoded = BlockCodec::deserialize(&encoded).expect("failed to deserialize block");

    // Assert
    assert_eq!(decoded, block);
}
