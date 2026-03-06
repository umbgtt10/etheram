// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::value_transfer::apply_value_transfers;
use etheram_node::state::storage::storage_mutation::StorageMutation;

#[test]
fn transfer_deducts_sender_balance_and_increments_nonce() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let accounts = BTreeMap::from([(sender, Account::new(500))]);
    let transaction = Transaction::transfer(sender, receiver, 200, 21_000, 1, 0);

    // Act
    let (updated, _) = apply_value_transfers(&[transaction], &accounts);

    // Assert
    assert_eq!(updated[&sender].balance, 300);
    assert_eq!(updated[&sender].nonce, 1);
}

#[test]
fn transfer_adds_to_receiver_balance() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let accounts = BTreeMap::from([(sender, Account::new(500)), (receiver, Account::new(100))]);
    let transaction = Transaction::transfer(sender, receiver, 200, 21_000, 1, 0);

    // Act
    let (updated, _) = apply_value_transfers(&[transaction], &accounts);

    // Assert
    assert_eq!(updated[&receiver].balance, 300);
}

#[test]
fn transfer_with_unknown_receiver_creates_account() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let accounts = BTreeMap::from([(sender, Account::new(100))]);
    let transaction = Transaction::transfer(sender, receiver, 50, 21_000, 1, 0);

    // Act
    let (updated, _) = apply_value_transfers(&[transaction], &accounts);

    // Assert
    assert_eq!(updated[&receiver].balance, 50);
}

#[test]
fn insufficient_balance_saturates_sender_to_zero() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let accounts = BTreeMap::from([(sender, Account::new(10))]);
    let transaction = Transaction::transfer(sender, receiver, 9_999, 21_000, 1, 0);

    // Act
    let (updated, _) = apply_value_transfers(&[transaction], &accounts);

    // Assert
    assert_eq!(updated[&sender].balance, 0);
}

#[test]
fn single_transfer_produces_two_mutations() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let accounts = BTreeMap::from([(sender, Account::new(500))]);
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 1, 0);

    // Act
    let (_, mutations) = apply_value_transfers(&[transaction], &accounts);

    // Assert
    assert_eq!(mutations.len(), 2);
    assert!(mutations
        .iter()
        .any(|m| matches!(m, StorageMutation::UpdateAccount(addr, _) if *addr == sender)));
    assert!(mutations
        .iter()
        .any(|m| matches!(m, StorageMutation::UpdateAccount(addr, _) if *addr == receiver)));
}

#[test]
fn two_transfers_produce_four_mutations() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let accounts = BTreeMap::from([(sender, Account::new(1_000))]);
    let tx1 = Transaction::transfer(sender, receiver, 100, 21_000, 1, 0);
    let tx2 = Transaction::transfer(sender, receiver, 200, 21_000, 1, 1);

    // Act
    let (_, mutations) = apply_value_transfers(&[tx1, tx2], &accounts);

    // Assert
    assert_eq!(mutations.len(), 4);
}
