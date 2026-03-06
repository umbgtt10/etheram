// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::execution::execution_engine::ExecutionEngine;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram_node::implementations::tiny_evm_engine::OPCODE_ADD;
use etheram_node::implementations::tiny_evm_engine::OPCODE_PUSH1;
use etheram_node::implementations::tiny_evm_engine::OPCODE_RETURN;
use etheram_node::implementations::tiny_evm_engine::OPCODE_SLOAD;
use etheram_node::implementations::tiny_evm_engine::OPCODE_SSTORE;
use etheram_node::implementations::tiny_evm_engine::OPCODE_STOP;
use etheram_node::implementations::value_transfer_engine::ValueTransferEngine;
use etheram_node::state::storage::storage_mutation::StorageMutation;

#[test]
fn execute_push_add_return_produces_no_contract_storage_mutation() {
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
    let transaction = Transaction::new(sender, contract, 0, 21_009, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert_eq!(mutations.len(), 2);
    assert!(!mutations
        .iter()
        .any(|mutation| matches!(mutation, StorageMutation::UpdateContractStorage { .. })));
}

#[test]
fn execute_sstore_emits_contract_storage_mutation() {
    // Arrange
    let sender = [3u8; 20];
    let contract = [4u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 0, 41_006, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, slot, value }
                if *address == contract && *slot == [0u8; 32] && value[31] == 0x2a
        )
    }));
}

#[test]
fn execute_sload_with_seeded_slot_stores_loaded_value_in_new_slot() {
    // Arrange
    let sender = [5u8; 20];
    let contract = [6u8; 20];
    let mut seeded_value = [0u8; 32];
    seeded_value[31] = 0x07;
    let mut slot_zero = [0u8; 32];
    slot_zero[31] = 0x00;
    let mut slot_one = [0u8; 32];
    slot_one[31] = 0x01;
    let bytecode = vec![
        OPCODE_PUSH1,
        0x00,
        OPCODE_SLOAD,
        OPCODE_PUSH1,
        0x01,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 0, 41_806, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::from([((contract, slot_zero), seeded_value)]);
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, slot, value }
                if *address == contract && *slot == slot_one && *value == seeded_value
        )
    }));
}

#[test]
fn execute_value_transfer_with_bytecode_updates_balances_and_contract_storage() {
    // Arrange
    let sender = [7u8; 20];
    let contract = [8u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 25, 41_006, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(10))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateAccount(address, account)
                if *address == sender && *account == Account { balance: 75, nonce: 1 }
        )
    }));
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateAccount(address, account)
                if *address == contract && *account == Account { balance: 35, nonce: 0 }
        )
    }));
    assert!(mutations
        .iter()
        .any(|mutation| matches!(mutation, StorageMutation::UpdateContractStorage { .. })));
}

#[test]
fn execute_same_block_with_different_engines_returns_different_mutations() {
    // Arrange
    let sender = [9u8; 20];
    let contract = [10u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 0, 41_006, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let value_transfer_engine = ValueTransferEngine;
    let tiny_evm_engine = TinyEvmEngine;

    // Act
    let vt_result = value_transfer_engine.execute(&block, &accounts, &contract_storage);
    let te_result = tiny_evm_engine.execute(&block, &accounts, &contract_storage);
    let value_transfer_mutations: Vec<_> = vt_result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();
    let tiny_evm_mutations: Vec<_> = te_result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert_eq!(value_transfer_mutations.len(), 2);
    assert_eq!(tiny_evm_mutations.len(), 3);
    assert!(!value_transfer_mutations
        .iter()
        .any(|mutation| matches!(mutation, StorageMutation::UpdateContractStorage { .. })));
    assert!(tiny_evm_mutations
        .iter()
        .any(|mutation| matches!(mutation, StorageMutation::UpdateContractStorage { .. })));
}

#[test]
fn execute_unknown_opcode_after_stop_does_not_change_behavior() {
    // Arrange
    let sender = [11u8; 20];
    let contract = [12u8; 20];
    let bytecode = vec![OPCODE_STOP, 0xff, OPCODE_PUSH1, 0x2a, OPCODE_RETURN];
    let transaction = Transaction::new(sender, contract, 0, 21_000, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert_eq!(mutations.len(), 2);
    assert!(!mutations
        .iter()
        .any(|mutation| matches!(mutation, StorageMutation::UpdateContractStorage { .. })));
}

#[test]
fn execute_empty_block_returns_no_mutations() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::new();
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

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
fn execute_unknown_opcode_mid_stream_halts_execution() {
    // Arrange
    let sender = [13u8; 20];
    let contract = [14u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x2a,
        0xff,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 0, 21_003, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert_eq!(
        result.transaction_results[0].status,
        TransactionStatus::InvalidOpcode
    );
    assert!(result.transaction_results[0].mutations.is_empty());
}

#[test]
fn execute_push1_at_end_of_bytecode_halts_gracefully() {
    // Arrange
    let sender = [15u8; 20];
    let contract = [16u8; 20];
    let bytecode = vec![OPCODE_PUSH1];
    let transaction = Transaction::new(sender, contract, 0, 21_003, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert_eq!(mutations.len(), 2);
    assert!(!mutations
        .iter()
        .any(|mutation| matches!(mutation, StorageMutation::UpdateContractStorage { .. })));
}

#[test]
fn execute_add_on_empty_stack_uses_zero() {
    // Arrange
    let sender = [17u8; 20];
    let contract = [18u8; 20];
    let bytecode = vec![OPCODE_ADD, OPCODE_PUSH1, 0x00, OPCODE_SSTORE, OPCODE_RETURN];
    let transaction = Transaction::new(sender, contract, 0, 41_006, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, slot, value }
                if *address == contract && *slot == [0u8; 32] && *value == [0u8; 32]
        )
    }));
}

#[test]
fn execute_sstore_on_empty_stack_uses_zero_slot_and_value() {
    // Arrange
    let sender = [19u8; 20];
    let contract = [20u8; 20];
    let bytecode = vec![OPCODE_SSTORE, OPCODE_RETURN];
    let transaction = Transaction::new(sender, contract, 0, 41_000, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, slot, value }
                if *address == contract && *slot == [0u8; 32] && *value == [0u8; 32]
        )
    }));
}

#[test]
fn execute_sload_on_empty_storage_pushes_zero() {
    // Arrange
    let sender = [21u8; 20];
    let contract = [22u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x05,
        OPCODE_SLOAD,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 0, 41_806, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, slot, value }
                if *address == contract && *slot == [0u8; 32] && *value == [0u8; 32]
        )
    }));
}

#[test]
fn execute_multiple_transactions_isolates_contract_storage() {
    // Arrange
    let sender = [23u8; 20];
    let contract_a = [24u8; 20];
    let contract_b = [25u8; 20];
    let bytecode_a = vec![
        OPCODE_PUSH1,
        0x0a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let bytecode_b = vec![
        OPCODE_PUSH1,
        0x0b,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let tx_a = Transaction::new(sender, contract_a, 0, 41_006, 1, 0, bytecode_a);
    let tx_b = Transaction::new(sender, contract_b, 0, 41_006, 1, 1, bytecode_b);
    let block = Block::new(0, 0, vec![tx_a, tx_b], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([
        (sender, Account::new(100)),
        (contract_a, Account::new(0)),
        (contract_b, Account::new(0)),
    ]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);
    let mutations: Vec<_> = result
        .transaction_results
        .into_iter()
        .flat_map(|r| r.mutations)
        .collect();

    // Assert
    let mut a_value = [0u8; 32];
    a_value[31] = 0x0a;
    let mut b_value = [0u8; 32];
    b_value[31] = 0x0b;
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, value, .. }
                if *address == contract_a && *value == a_value
        )
    }));
    assert!(mutations.iter().any(|mutation| {
        matches!(
            mutation,
            StorageMutation::UpdateContractStorage { address, value, .. }
                if *address == contract_b && *value == b_value
        )
    }));
}

#[test]
fn execute_single_transaction_returns_success_status() {
    // Arrange
    let sender = [26u8; 20];
    let contract = [27u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let transaction = Transaction::new(sender, contract, 0, 41_006, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
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
}

#[test]
fn execute_single_transaction_returns_intrinsic_gas_used() {
    // Arrange
    let sender = [28u8; 20];
    let contract = [29u8; 20];
    let bytecode = vec![OPCODE_RETURN];
    let transaction = Transaction::new(sender, contract, 0, 21_000, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results[0].gas_used, 21_000);
}

#[test]
fn execute_sstore_existing_slot_uses_reset_gas() {
    // Arrange
    let sender = [30u8; 20];
    let contract = [31u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0xff,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let slot_zero = [0u8; 32];
    let mut existing_value = [0u8; 32];
    existing_value[31] = 0x01;
    let reset_gas = 21_000 + 3 + 3 + 5_000;
    let transaction = Transaction::new(sender, contract, 0, reset_gas, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::from([((contract, slot_zero), existing_value)]);
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert_eq!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    );
    assert_eq!(result.transaction_results[0].gas_used, reset_gas);
}

#[test]
fn execute_sstore_existing_slot_with_set_gas_runs_out_of_gas() {
    // Arrange
    let sender = [32u8; 20];
    let contract = [33u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0xff,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let slot_zero = [0u8; 32];
    let mut existing_value = [0u8; 32];
    existing_value[31] = 0x01;
    let only_set_gas = 21_000 + 3 + 3 + 5_000 - 1;
    let transaction = Transaction::new(sender, contract, 0, only_set_gas, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::from([((contract, slot_zero), existing_value)]);
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 1);
    assert_eq!(
        result.transaction_results[0].status,
        TransactionStatus::OutOfGas
    );
}

#[test]
fn execute_value_transfer_with_bytecode_oog_reverts_entire_transaction() {
    // Arrange
    let sender = [34u8; 20];
    let contract = [35u8; 20];
    let bytecode = vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let too_little_gas = 21_003;
    let transaction = Transaction::new(sender, contract, 50, too_little_gas, 1, 0, bytecode);
    let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(10))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

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

#[test]
fn execute_oog_second_tx_does_not_undo_first_tx_contract_storage() {
    // Arrange
    let sender = [36u8; 20];
    let contract = [37u8; 20];
    let bytecode_ok = vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let bytecode_oog = vec![
        OPCODE_PUSH1,
        0x01,
        OPCODE_PUSH1,
        0x01,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let sufficient_gas = 21_000 + 3 + 3 + 20_000;
    let insufficient_gas = 21_003;
    let tx1 = Transaction::new(sender, contract, 0, sufficient_gas, 1, 0, bytecode_ok);
    let tx2 = Transaction::new(sender, contract, 0, insufficient_gas, 1, 1, bytecode_oog);
    let block = Block::new(0, 0, vec![tx1, tx2], [0u8; 32], BLOCK_GAS_LIMIT);
    let accounts = BTreeMap::from([(sender, Account::new(100)), (contract, Account::new(0))]);
    let contract_storage = BTreeMap::new();
    let engine = TinyEvmEngine;

    // Act
    let result = engine.execute(&block, &accounts, &contract_storage);

    // Assert
    assert_eq!(result.transaction_results.len(), 2);
    assert_eq!(
        result.transaction_results[0].status,
        TransactionStatus::Success
    );
    assert_eq!(
        result.transaction_results[1].status,
        TransactionStatus::OutOfGas
    );
    assert!(!result.transaction_results[0].mutations.is_empty());
    assert!(result.transaction_results[1].mutations.is_empty());
}
