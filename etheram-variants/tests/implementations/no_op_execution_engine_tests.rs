// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::execution::execution_engine::ExecutionEngine;
use etheram_variants::implementations::no_op_execution_engine::NoOpExecutionEngine;
use std::collections::BTreeMap;

#[test]
fn execute_empty_block_returns_no_mutations() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let accounts = BTreeMap::new();
    let contract_storage = BTreeMap::new();
    let engine = NoOpExecutionEngine;

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
fn execute_block_with_transaction_returns_no_mutations() {
    // Arrange
    let sender = [1u8; 20];
    let receiver = [2u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = NoOpExecutionEngine;

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
fn execute_any_block_returns_empty_transaction_results() {
    // Arrange
    let sender = [3u8; 20];
    let receiver = [4u8; 20];
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(1_000)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = NoOpExecutionEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert!(result.transaction_results.is_empty());
}
