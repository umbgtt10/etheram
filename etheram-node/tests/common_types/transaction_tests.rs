// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::transaction::Transaction;

#[test]
fn cmp_higher_gas_price_is_greater() {
    // Arrange
    let low = Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 1, 0);
    let high = Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 2, 0);

    // Act
    let ordering = high.cmp(&low);

    // Assert
    assert!(ordering.is_gt());
}

#[test]
fn cmp_equal_gas_lower_nonce_is_greater() {
    // Arrange
    let lower_nonce = Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 5, 0);
    let higher_nonce = Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 5, 7);

    // Act
    let ordering = lower_nonce.cmp(&higher_nonce);

    // Assert
    assert!(ordering.is_gt());
}

#[test]
fn cmp_equal_gas_nonce_lower_sender_is_greater() {
    // Arrange
    let lower_sender = Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 5, 0);
    let higher_sender = Transaction::transfer([2u8; 20], [2u8; 20], 1, 21_000, 5, 0);

    // Act
    let ordering = lower_sender.cmp(&higher_sender);

    // Assert
    assert!(ordering.is_gt());
}
