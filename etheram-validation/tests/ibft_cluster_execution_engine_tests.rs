// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::build_block_with_commitments;
use crate::common::ibft_cluster_test_helpers::finalize_round_with_block;
use crate::common::ibft_cluster_test_helpers::validators;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram::common_types::types::Address;
use etheram::common_types::types::Hash;
use etheram::execution::execution_engine::ExecutionEngine;
use etheram::execution::execution_result::ExecutionResult;
use etheram::execution::transaction_result::TransactionResult;
use etheram::execution::transaction_result::TransactionStatus;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram_etheram_validation::ibft_cluster::IbftCluster;
use etheram_etheram_variants::implementations::no_op_execution_engine::NoOpExecutionEngine;
use etheram_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram_etheram_variants::implementations::tiny_evm_engine::OPCODE_PUSH1;
use etheram_etheram_variants::implementations::tiny_evm_engine::OPCODE_RETURN;
use etheram_etheram_variants::implementations::tiny_evm_engine::OPCODE_SSTORE;
use etheram_etheram_variants::implementations::value_transfer_engine::ValueTransferEngine;
use std::collections::BTreeMap;

fn finalize_first_block_with_transaction(
    cluster: &mut IbftCluster,
    tx: Transaction,
    from: [u8; 20],
    engine: &dyn ExecutionEngine,
) {
    let genesis_accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let state_root = etheram::common_types::state_root::compute_state_root(&genesis_accounts);
    let contract_storage = BTreeMap::new();
    let proposed_block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &genesis_accounts,
        &contract_storage,
        engine,
    );

    finalize_round_with_block(cluster, 0, 0, 0, &proposed_block);
}

fn word(value: u8) -> Hash {
    let mut hash = [0u8; 32];
    hash[31] = value;
    hash
}

#[test]
fn cluster_value_transfer_engine_commit_updates_balances_without_contract_storage() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    finalize_first_block_with_transaction(&mut cluster, tx, from, &ValueTransferEngine);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(
        cluster.node_account(0, from).map(|account| account.balance),
        Some(900)
    );
    assert_eq!(
        cluster.node_account(0, to).map(|account| account.balance),
        Some(100)
    );
    assert_eq!(cluster.node_contract_storage(0, to, word(0)), None);
}

#[test]
fn cluster_engine_swap_changes_contract_storage_effects_for_same_transaction() {
    // Arrange
    let from = [3u8; 20];
    let contract = [4u8; 20];
    let tx = Transaction::new(
        from,
        contract,
        0,
        41_006,
        0,
        vec![
            OPCODE_PUSH1,
            0x2a,
            OPCODE_PUSH1,
            0x00,
            OPCODE_SSTORE,
            OPCODE_RETURN,
        ],
    );
    let mut value_transfer_cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    let mut tiny_evm_cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(TinyEvmEngine)
        });
    value_transfer_cluster.submit_request(0, 10, ClientRequest::SubmitTransaction(tx.clone()));
    value_transfer_cluster.drain(0);
    tiny_evm_cluster.submit_request(0, 11, ClientRequest::SubmitTransaction(tx.clone()));
    tiny_evm_cluster.drain(0);

    // Act
    finalize_first_block_with_transaction(
        &mut value_transfer_cluster,
        tx.clone(),
        from,
        &ValueTransferEngine,
    );
    finalize_first_block_with_transaction(&mut tiny_evm_cluster, tx, from, &TinyEvmEngine);

    // Assert
    assert_eq!(value_transfer_cluster.node_height(0), 1);
    assert_eq!(tiny_evm_cluster.node_height(0), 1);
    assert_eq!(
        value_transfer_cluster.node_contract_storage(0, contract, word(0)),
        None
    );
    assert_eq!(
        tiny_evm_cluster.node_contract_storage(0, contract, word(0)),
        Some(word(0x2a))
    );
}

#[test]
fn cluster_tiny_evm_engine_sstore_transaction_persists_contract_storage() {
    // Arrange
    let from = [5u8; 20];
    let contract = [6u8; 20];
    let tx = Transaction::new(
        from,
        contract,
        0,
        41_006,
        0,
        vec![
            OPCODE_PUSH1,
            0x2a,
            OPCODE_PUSH1,
            0x00,
            OPCODE_SSTORE,
            OPCODE_RETURN,
        ],
    );
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(TinyEvmEngine)
        });
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    finalize_first_block_with_transaction(&mut cluster, tx, from, &TinyEvmEngine);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(
        cluster.node_contract_storage(0, contract, word(0)),
        Some(word(0x2a))
    );
}

#[test]
fn cluster_engine_swap_does_not_affect_value_transfer_balances() {
    // Arrange
    let from = [7u8; 20];
    let to = [8u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut vt_cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    let mut evm_cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(TinyEvmEngine)
        });
    vt_cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    vt_cluster.drain(0);
    evm_cluster.submit_request(0, 2, ClientRequest::SubmitTransaction(tx.clone()));
    evm_cluster.drain(0);

    // Act
    finalize_first_block_with_transaction(&mut vt_cluster, tx.clone(), from, &ValueTransferEngine);
    finalize_first_block_with_transaction(&mut evm_cluster, tx, from, &TinyEvmEngine);

    // Assert
    assert_eq!(
        vt_cluster.node_account(0, from).map(|a| a.balance),
        Some(900)
    );
    assert_eq!(
        evm_cluster.node_account(0, from).map(|a| a.balance),
        Some(900)
    );
    assert_eq!(vt_cluster.node_account(0, to).map(|a| a.balance), Some(100));
    assert_eq!(
        evm_cluster.node_account(0, to).map(|a| a.balance),
        Some(100)
    );
}

#[test]
fn cluster_noop_engine_commit_does_not_update_accounts_or_storage() {
    // Arrange
    let from = [9u8; 20];
    let to = [10u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(NoOpExecutionEngine)
        });
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    finalize_first_block_with_transaction(&mut cluster, tx, from, &NoOpExecutionEngine);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(
        cluster.node_account(0, from).map(|a| a.balance),
        Some(1_000)
    );
    assert_eq!(cluster.node_account(0, to), None);
    assert_eq!(cluster.node_contract_storage(0, to, word(0)), None);
}

struct OutOfGasEngine;

impl ExecutionEngine for OutOfGasEngine {
    fn execute(
        &self,
        block: &Block,
        _accounts: &BTreeMap<Address, Account>,
        _contract_storage: &BTreeMap<(Address, Hash), Hash>,
    ) -> ExecutionResult {
        let transaction_results = block
            .transactions
            .iter()
            .map(|tx| TransactionResult {
                from: tx.from,
                status: TransactionStatus::OutOfGas,
                gas_used: 21_000,
                mutations: Vec::new(),
            })
            .collect();
        ExecutionResult {
            transaction_results,
        }
    }
}

#[test]
fn cluster_out_of_gas_engine_does_not_apply_account_mutations() {
    // Arrange
    let from = [11u8; 20];
    let to = [12u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(OutOfGasEngine)
        });
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    finalize_first_block_with_transaction(&mut cluster, tx, from, &OutOfGasEngine);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(
        cluster.node_account(0, from).map(|a| a.balance),
        Some(1_000)
    );
    assert_eq!(cluster.node_account(0, to), None);
}
