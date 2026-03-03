// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_ADD;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_PUSH1;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_RETURN;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_SSTORE;
use barechain_etheram_variants::implementations::tiny_evm_gas::GAS_ADD;
use barechain_etheram_variants::implementations::tiny_evm_gas::GAS_PUSH1;
use barechain_etheram_variants::implementations::tiny_evm_gas::GAS_SSTORE_SET;
use barechain_etheram_variants::implementations::tiny_evm_gas::INTRINSIC_GAS;
use barechain_etheram_variants::implementations::value_transfer_engine::ValueTransferEngine;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram::execution::execution_engine::ExecutionEngine;
use etheram::execution::transaction_result::TransactionStatus;
use std::collections::BTreeMap;

#[test]
fn execute_exact_gas_succeeds() {
    // Arrange
    let sender = [1u8; 20];
    let contract = [2u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x02,
        OPCODE_ADD,
        OPCODE_RETURN,
    ];
    let opcode_cost = GAS_PUSH1 + GAS_PUSH1 + GAS_ADD;
    let gas_limit = INTRINSIC_GAS + opcode_cost;
    let transaction = Transaction::new(sender, contract, 0, gas_limit, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
    assert_eq!(result.transaction_results[0].gas_used, gas_limit);
}

#[test]
fn execute_one_gas_short_reverts() {
    // Arrange
    let sender = [3u8; 20];
    let contract = [4u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x02,
        OPCODE_ADD,
        OPCODE_RETURN,
    ];
    let exact_cost = INTRINSIC_GAS + GAS_PUSH1 + GAS_PUSH1 + GAS_ADD;
    let gas_limit = exact_cost - 1;
    let transaction = Transaction::new(sender, contract, 0, gas_limit, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    ));
    assert!(result.transaction_results[0].mutations.is_empty());
}

#[test]
fn execute_sstore_out_of_gas_reverts_all_mutations() {
    // Arrange
    let sender = [5u8; 20];
    let contract = [6u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let gas_limit = INTRINSIC_GAS + GAS_PUSH1 + GAS_PUSH1;
    let transaction = Transaction::new(sender, contract, 100, gas_limit, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(200)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    ));
    assert!(result.transaction_results[0].mutations.is_empty());
}

#[test]
fn execute_value_transfer_insufficient_intrinsic_gas_reverts() {
    // Arrange
    let sender = [7u8; 20];
    let receiver = [8u8; 20];
    let gas_limit = INTRINSIC_GAS - 1;
    let transaction = Transaction::transfer(sender, receiver, 50, gas_limit, 0);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(200)), (receiver, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = ValueTransferEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    ));
    assert_eq!(result.transaction_results[0].gas_used, gas_limit);
    assert!(result.transaction_results[0].mutations.is_empty());
}

#[test]
fn execute_multiple_transactions_partial_revert() {
    // Arrange
    let sender = [9u8; 20];
    let receiver = [10u8; 20];
    let tx_success = Transaction::transfer(sender, receiver, 50, INTRINSIC_GAS, 0);
    let tx_out_of_gas = Transaction::transfer(sender, receiver, 50, INTRINSIC_GAS - 1, 1);
    let block = Block::new(0, 0, vec![tx_success, tx_out_of_gas], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(500)), (receiver, Account::new(0))]);
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
    assert!(!result.transaction_results[0].mutations.is_empty());
    assert!(matches!(
        result.transaction_results[1].status,
        TransactionStatus::OutOfGas
    ));
    assert!(result.transaction_results[1].mutations.is_empty());
}

#[test]
fn gas_used_reflects_actual_consumption() {
    // Arrange
    let sender = [11u8; 20];
    let contract = [12u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let exact_cost = INTRINSIC_GAS + GAS_PUSH1 + GAS_PUSH1 + GAS_SSTORE_SET;
    let gas_limit = exact_cost + 10_000;
    let transaction = Transaction::new(sender, contract, 0, gas_limit, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32]);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
    assert_eq!(result.transaction_results[0].gas_used, exact_cost);
}
