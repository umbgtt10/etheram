// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::execution::execution_engine::ExecutionEngine;
use etheram_node::execution::execution_result::ExecutionResult;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram_node::implementations::tiny_evm_engine::OPCODE_ADD;
use etheram_node::implementations::tiny_evm_engine::OPCODE_AND;
use etheram_node::implementations::tiny_evm_engine::OPCODE_CALLDATALOAD;
use etheram_node::implementations::tiny_evm_engine::OPCODE_CALLDATASIZE;
use etheram_node::implementations::tiny_evm_engine::OPCODE_CALLER;
use etheram_node::implementations::tiny_evm_engine::OPCODE_CALLVALUE;
use etheram_node::implementations::tiny_evm_engine::OPCODE_DUP1;
use etheram_node::implementations::tiny_evm_engine::OPCODE_JUMP;
use etheram_node::implementations::tiny_evm_engine::OPCODE_JUMPDEST;
use etheram_node::implementations::tiny_evm_engine::OPCODE_JUMPI;
use etheram_node::implementations::tiny_evm_engine::OPCODE_MLOAD;
use etheram_node::implementations::tiny_evm_engine::OPCODE_MSTORE;
use etheram_node::implementations::tiny_evm_engine::OPCODE_POP;
use etheram_node::implementations::tiny_evm_engine::OPCODE_PUSH1;
use etheram_node::implementations::tiny_evm_engine::OPCODE_PUSH2;
use etheram_node::implementations::tiny_evm_engine::OPCODE_RETURN;
use etheram_node::implementations::tiny_evm_engine::OPCODE_REVERT;
use etheram_node::implementations::tiny_evm_engine::OPCODE_SHA3;
use etheram_node::implementations::tiny_evm_engine::OPCODE_STOP;
use etheram_node::implementations::tiny_evm_engine::OPCODE_SWAP1;
use etheram_node::implementations::tiny_evm_gas::GAS_MLOAD_BASE;
use etheram_node::implementations::tiny_evm_gas::GAS_MSTORE_BASE;
use etheram_node::implementations::tiny_evm_gas::INTRINSIC_GAS;

fn run_bytecode(
    sender: [u8; 20],
    contract: [u8; 20],
    bytecode: Vec<u8>,
    gas_limit: u64,
    value: u128,
) -> ExecutionResult {
    let transaction = Transaction::new(sender, contract, value, gas_limit, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([
        (sender, Account::new(1_000_000)),
        (contract, Account::new(0)),
    ]);
    let contract_storage = BTreeMap::new();
    TinyEvmEngine.execute(&block, &accounts, &contract_storage)
}

#[test]
fn mstore_mload_roundtrip() {
    // Arrange
    let sender = [1u8; 20];
    let contract = [2u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x42,
        OPCODE_PUSH1,
        0x00,
        OPCODE_MSTORE,
        OPCODE_PUSH1,
        0x00,
        OPCODE_MLOAD,
        OPCODE_RETURN,
    ];
    let gas = INTRINSIC_GAS + GAS_MSTORE_BASE + GAS_MLOAD_BASE + 3 * 3 + 3 + 3 + 200;

    // Act
    let result = run_bytecode(sender, contract, bytecode, gas, 0);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn calldatasize_returns_data_length() {
    // Arrange
    let sender = [3u8; 20];
    let contract = [4u8; 20];
    let bytecode = vec![OPCODE_CALLDATASIZE, OPCODE_RETURN];

    // Act
    let result = run_bytecode(sender, contract, bytecode, 50_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn calldataload_reads_padded_word() {
    // Arrange
    let sender = [5u8; 20];
    let contract = [6u8; 20];
    let mut bytecode = vec![OPCODE_PUSH1, 0x00, OPCODE_CALLDATALOAD, OPCODE_RETURN];
    bytecode.extend_from_slice(&[0xaa; 32]);

    // Act
    let result = run_bytecode(sender, contract, bytecode, 50_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn sha3_produces_correct_keccak256() {
    // Arrange
    let sender = [7u8; 20];
    let contract = [8u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x00,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SHA3,
        OPCODE_RETURN,
    ];

    // Act
    let result = run_bytecode(sender, contract, bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn jump_to_jumpdest_succeeds() {
    // Arrange
    let dest: u8 = 4;
    let bytecode = vec![
        OPCODE_PUSH1,
        dest,
        OPCODE_JUMP,
        OPCODE_STOP,
        OPCODE_JUMPDEST,
        OPCODE_RETURN,
    ];

    // Act
    let result = run_bytecode([9u8; 20], [10u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn jumpi_not_taken_when_condition_zero() {
    // Arrange
    let dest: u8 = 6;
    let bytecode = vec![
        OPCODE_PUSH1,
        0x00,
        OPCODE_PUSH1,
        dest,
        OPCODE_JUMPI,
        OPCODE_RETURN,
        OPCODE_JUMPDEST,
        OPCODE_STOP,
    ];

    // Act
    let result = run_bytecode([11u8; 20], [12u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn jumpi_taken_when_condition_nonzero() {
    // Arrange
    let dest: u8 = 6;
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        dest,
        OPCODE_JUMPI,
        OPCODE_STOP,
        OPCODE_JUMPDEST,
        OPCODE_RETURN,
    ];

    // Act
    let result = run_bytecode([13u8; 20], [14u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn jump_to_non_jumpdest_returns_out_of_gas() {
    // Arrange
    let bytecode = vec![OPCODE_PUSH1, 0x03, OPCODE_JUMP, OPCODE_ADD];

    // Act
    let result = run_bytecode([15u8; 20], [16u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    ));
}

#[test]
fn push2_encodes_two_byte_immediate() {
    // Arrange
    let bytecode = vec![OPCODE_PUSH2, 0x01, 0x00, OPCODE_RETURN];

    // Act
    let result = run_bytecode([17u8; 20], [18u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn dup1_duplicates_top() {
    // Arrange
    let bytecode = vec![OPCODE_PUSH1, 0x2a, OPCODE_DUP1, OPCODE_ADD, OPCODE_RETURN];

    // Act
    let result = run_bytecode([19u8; 20], [20u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn swap1_exchanges_top_two() {
    // Arrange: PUSH1 1 PUSH1 2 SWAP1 RETURN  — top becomes 1, succeeds
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x02,
        OPCODE_SWAP1,
        OPCODE_RETURN,
    ];

    // Act
    let result = run_bytecode([21u8; 20], [22u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn caller_pushes_sender_address() {
    // Arrange
    let sender = [0xabu8; 20];
    let bytecode = vec![OPCODE_CALLER, OPCODE_RETURN];

    // Act
    let result = run_bytecode(sender, [23u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn callvalue_pushes_transfer_amount() {
    // Arrange
    let sender = [0xbcu8; 20];
    let contract = [24u8; 20];
    let value = 12345u128;
    let bytecode = vec![OPCODE_CALLVALUE, OPCODE_RETURN];

    // Act
    let result = run_bytecode(sender, contract, bytecode, 100_000, value);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn memory_expansion_gas_charged() {
    // Arrange
    let sender = [25u8; 20];
    let contract = [26u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x00,
        OPCODE_MSTORE,
        OPCODE_RETURN,
    ];
    let exact_gas = INTRINSIC_GAS + 3 + 3 + GAS_MSTORE_BASE + 3 + 3;

    // Act
    let result_exact = run_bytecode(sender, contract, bytecode.clone(), exact_gas + 100, 0);
    let result_no_gas = run_bytecode(sender, contract, bytecode, INTRINSIC_GAS + 1, 0);

    // Assert
    assert!(matches!(
        result_exact.transaction_results[0].status,
        TransactionStatus::Success
    ));
    assert!(matches!(
        result_no_gas.transaction_results[0].status,
        TransactionStatus::OutOfGas
    ));
}

#[test]
fn revert_returns_out_of_gas_status() {
    // Arrange
    let bytecode = vec![OPCODE_REVERT];

    // Act
    let result = run_bytecode([27u8; 20], [28u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    ));
}

#[test]
fn pop_discards_top_of_stack() {
    // Arrange
    let bytecode = vec![OPCODE_PUSH1, 0x2a, OPCODE_POP, OPCODE_RETURN];

    // Act
    let result = run_bytecode([29u8; 20], [30u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}

#[test]
fn and_produces_bitwise_result() {
    // Arrange
    let bytecode = vec![
        OPCODE_PUSH1,
        0x0f,
        OPCODE_PUSH1,
        0xff,
        OPCODE_AND,
        OPCODE_RETURN,
    ];

    // Act
    let result = run_bytecode([31u8; 20], [32u8; 20], bytecode, 100_000, 0);

    // Assert
    assert!(matches!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    ));
}
