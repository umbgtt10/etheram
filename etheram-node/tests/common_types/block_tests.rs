// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;

#[test]
fn compute_hash_same_block_twice_returns_same_hash() {
    // Arrange
    let block = Block::new(
        3,
        2,
        vec![Transaction::transfer(
            [1u8; 20], [2u8; 20], 10, 21_000, 1, 0,
        )],
        [9u8; 32],
    );

    // Act
    let first_hash = block.compute_hash();
    let second_hash = block.compute_hash();

    // Assert
    assert_eq!(first_hash, second_hash);
}

#[test]
fn compute_hash_different_height_returns_different_hash() {
    // Arrange
    let block_zero = Block::new(0, 1, vec![], [0u8; 32]);
    let block_one = Block::new(1, 1, vec![], [0u8; 32]);

    // Act
    let zero_hash = block_zero.compute_hash();
    let one_hash = block_one.compute_hash();

    // Assert
    assert_ne!(zero_hash, one_hash);
}

#[test]
fn compute_hash_different_transactions_returns_different_hash() {
    // Arrange
    let tx_a = Transaction::transfer([1u8; 20], [2u8; 20], 7, 21_000, 1, 0);
    let tx_b = Transaction::transfer([3u8; 20], [4u8; 20], 11, 21_000, 1, 1);
    let left_block = Block::new(5, 4, vec![tx_a.clone(), tx_b.clone()], [6u8; 32]);
    let right_block = Block::new(5, 4, vec![tx_b, tx_a], [6u8; 32]);

    // Act
    let left_hash = left_block.compute_hash();
    let right_hash = right_block.compute_hash();

    // Assert
    assert_ne!(left_hash, right_hash);
}

#[test]
fn compute_hash_different_post_state_root_returns_different_hash() {
    // Arrange
    let mut block_a = Block::new(0, 1, vec![], [0u8; 32]);
    let mut block_b = Block::new(0, 1, vec![], [0u8; 32]);
    block_a.post_state_root = [1u8; 32];
    block_b.post_state_root = [2u8; 32];

    // Act
    let hash_a = block_a.compute_hash();
    let hash_b = block_b.compute_hash();

    // Assert
    assert_ne!(hash_a, hash_b);
}

#[test]
fn compute_hash_different_receipts_root_returns_different_hash() {
    // Arrange
    let mut block_a = Block::new(0, 1, vec![], [0u8; 32]);
    let mut block_b = Block::new(0, 1, vec![], [0u8; 32]);
    block_a.receipts_root = [1u8; 32];
    block_b.receipts_root = [2u8; 32];

    // Act
    let hash_a = block_a.compute_hash();
    let hash_b = block_b.compute_hash();

    // Assert
    assert_ne!(hash_a, hash_b);
}
