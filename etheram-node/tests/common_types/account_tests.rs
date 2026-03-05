// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::account::Account;

#[test]
fn new_sets_balance_and_nonce_zero() {
    // Arrange
    let balance = 1_000;

    // Act
    let account = Account::new(balance);

    // Assert
    assert_eq!(account.balance, balance);
    assert_eq!(account.nonce, 0);
}

#[test]
fn empty_returns_zero_balance_and_zero_nonce() {
    // Arrange & Act
    let account = Account::empty();

    // Assert
    assert_eq!(account.balance, 0);
    assert_eq!(account.nonce, 0);
}

#[test]
fn new_different_balances_are_not_equal() {
    // Arrange
    let a = Account::new(100);
    let b = Account::new(200);

    // Act & Assert
    assert_ne!(a, b);
}

#[test]
fn new_same_balance_are_equal() {
    // Arrange
    let a = Account::new(500);
    let b = Account::new(500);

    // Act & Assert
    assert_eq!(a, b);
}
