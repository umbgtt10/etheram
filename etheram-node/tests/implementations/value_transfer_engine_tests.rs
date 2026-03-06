// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::execution::execution_engine::ExecutionEngine;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node::implementations::value_transfer_engine::ValueTransferEngine;
use etheram_node::state::storage::storage_mutation::StorageMutation;

#[test]
fn execute_single_transfer_returns_sender_and_receiver_account_updates() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 1, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(10))]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert_eq!(mutations.len(), 2);
    assert!(matches!(
        mutations.first(),
        Some(StorageMutation::UpdateAccount(address, account))
            if *address == sender && *account == Account { balance: 900, nonce: 1 }
    ));
    assert!(matches!(
        mutations.get(1),
        Some(StorageMutation::UpdateAccount(address, account))
            if *address == receiver && *account == Account { balance: 110, nonce: 0 }
    ));
}

#[test]
fn execute_receiver_near_max_balance_saturates_addition() {
    // Arrange
    let sender = [3u8; 20];
    let receiver = [4u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 1, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([
        (sender, Account::new(1_000)),
        (
            receiver,
            Account {
                balance: u128::MAX - 50,
                nonce: 0,
            },
        ),
    ]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(matches!(
        mutations.get(1),
        Some(StorageMutation::UpdateAccount(_, Account { balance, nonce }))
            if *balance == u128::MAX && *nonce == 0
    ));
}

#[test]
fn execute_two_transfers_same_receiver_accumulates_over_working_state() {
    // Arrange
    let sender_a = [5u8; 20];
    let sender_b = [6u8; 20];
    let receiver = [7u8; 20];
    let tx_a = Transaction::transfer(sender_a, receiver, 300, 21_000, 1, 0);
    let tx_b = Transaction::transfer(sender_b, receiver, 200, 21_000, 1, 0);
    let block = Block::new(0, 0, vec![tx_a, tx_b], [0u8; 32]);
    let accounts = BTreeMap::from([
        (sender_a, Account::new(1_000)),
        (sender_b, Account::new(500)),
        (receiver, Account::new(0)),
    ]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert_eq!(mutations.len(), 4);
    assert!(matches!(
        mutations.get(3),
        Some(StorageMutation::UpdateAccount(address, account))
            if *address == receiver && *account == Account { balance: 500, nonce: 0 }
    ));
}

#[test]
fn execute_empty_block_returns_no_mutations() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let accounts = BTreeMap::new();
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.is_empty());
}

#[test]
fn execute_single_transfer_returns_success_status() {
    // Arrange
    let sender = [8u8; 20];
    let receiver = [9u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 1, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn execute_single_transfer_returns_intrinsic_gas_used() {
    // Arrange
    let sender = [10u8; 20];
    let receiver = [11u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 1, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results[0].gas_used, 21_000);
}

#[test]
fn execute_two_transactions_in_block_returns_two_results() {
    // Arrange
    let sender_a = [12u8; 20];
    let sender_b = [13u8; 20];
    let receiver = [14u8; 20];
    let tx_a = Transaction::transfer(sender_a, receiver, 100, 21_000, 1, 0);
    let tx_b = Transaction::transfer(sender_b, receiver, 50, 21_000, 1, 0);
    let block = Block::new(0, 0, vec![tx_a, tx_b], [0u8; 32]);
    let accounts = BTreeMap::from([
        (sender_a, Account::new(500)),
        (sender_b, Account::new(200)),
        (receiver, Account::new(0)),
    ]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 2);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
    assert!(matches!(
        result.transaction_results[1].status,
        TransactionStatus::Success
    ));
}

#[test]
fn execute_gas_limit_zero_returns_out_of_gas() {
    // Arrange
    let sender = [15u8; 20];
    let receiver = [16u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 0, 1, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert_eq!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    );
    assert_eq!(result.transaction_results[0].gas_used, 0);
    assert!(result.transaction_results[0].mutations.is_empty());
}

#[test]
fn execute_gas_limit_below_intrinsic_returns_out_of_gas_preserving_balance() {
    // Arrange
    let sender = [17u8; 20];
    let receiver = [18u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 20_999, 1, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert_eq!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    );
    assert!(result.transaction_results[0].mutations.is_empty());
}
