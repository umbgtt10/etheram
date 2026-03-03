// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::build_block_with_commitments;
use crate::common::ibft_cluster_test_helpers::finalize_round_with_block;
use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use barechain_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_PUSH1;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_RETURN;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_SSTORE;
use barechain_etheram_variants::implementations::tiny_evm_gas::GAS_PUSH1;
use barechain_etheram_variants::implementations::tiny_evm_gas::GAS_SSTORE_SET;
use barechain_etheram_variants::implementations::tiny_evm_gas::INTRINSIC_GAS;
use barechain_etheram_variants::implementations::value_transfer_engine::ValueTransferEngine;
use etheram::common_types::account::Account;
use etheram::common_types::state_root::compute_state_root;
use etheram::common_types::transaction::Transaction;
use etheram::common_types::types::Hash;
use etheram::execution::execution_engine::ExecutionEngine;
use etheram::execution::transaction_result::TransactionStatus;
use etheram::incoming::external_interface::client_request::ClientRequest;
use std::collections::BTreeMap;

fn finalize_block(
    cluster: &mut IbftCluster,
    transactions: Vec<Transaction>,
    genesis_accounts: BTreeMap<[u8; 20], Account>,
    engine: &dyn ExecutionEngine,
) {
    let state_root = compute_state_root(&genesis_accounts);
    let contract_storage = BTreeMap::new();
    let proposed_block = build_block_with_commitments(
        0,
        0,
        transactions,
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
fn cluster_out_of_gas_transaction_reverts_account_state() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, INTRINSIC_GAS - 1, 0);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    finalize_block(
        &mut cluster,
        vec![tx],
        genesis_accounts,
        &ValueTransferEngine,
    );

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(
        cluster.node_account(0, from).map(|a| a.balance),
        Some(1_000)
    );
    assert_eq!(cluster.node_account(0, to), None);
}

#[test]
fn cluster_mixed_block_partial_gas_failure() {
    // Arrange
    let from1 = [3u8; 20];
    let from2 = [5u8; 20];
    let to = [4u8; 20];
    let tx1 = Transaction::transfer(from1, to, 300, INTRINSIC_GAS, 0);
    let tx2 = Transaction::transfer(from2, to, 50, INTRINSIC_GAS - 1, 0);
    let genesis_accounts =
        BTreeMap::from([(from1, Account::new(1_000)), (from2, Account::new(500))]);
    let mut cluster = IbftCluster::new_with_execution_engine_factory(
        validators(),
        vec![(from1, 1_000), (from2, 500)],
        || Box::new(ValueTransferEngine),
    );
    cluster.submit_request(0, 2, ClientRequest::SubmitTransaction(tx1.clone()));
    cluster.submit_request(0, 3, ClientRequest::SubmitTransaction(tx2.clone()));
    cluster.drain(0);

    // Act
    finalize_block(
        &mut cluster,
        vec![tx1, tx2],
        genesis_accounts,
        &ValueTransferEngine,
    );

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_account(0, from1).map(|a| a.balance), Some(700));
    assert_eq!(cluster.node_account(0, from2).map(|a| a.balance), Some(500));
    assert_eq!(cluster.node_account(0, to).map(|a| a.balance), Some(300));
}

#[test]
fn cluster_gas_exactly_sufficient_succeeds() {
    // Arrange
    let from = [5u8; 20];
    let contract = [6u8; 20];
    let exact_gas = INTRINSIC_GAS + 2 * GAS_PUSH1 + GAS_SSTORE_SET;
    let tx = Transaction::new(
        from,
        contract,
        0,
        exact_gas,
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
    let genesis_accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(TinyEvmEngine)
        });
    cluster.submit_request(0, 3, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);

    // Act
    finalize_block(&mut cluster, vec![tx], genesis_accounts, &TinyEvmEngine);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(
        cluster.node_account(0, from).map(|a| a.balance),
        Some(1_000)
    );
    assert_eq!(
        cluster.node_contract_storage(0, contract, word(0)),
        Some(word(0x2a))
    );
}

#[test]
fn cluster_receipts_on_commit_match_tx_execution_results() {
    // Arrange
    let from1: [u8; 20] = [3u8; 20];
    let from2: [u8; 20] = [5u8; 20];
    let to: [u8; 20] = [9u8; 20];
    let tx1 = Transaction::transfer(from1, to, 300, INTRINSIC_GAS, 0);
    let tx2 = Transaction::transfer(from2, to, 50, INTRINSIC_GAS - 1, 0);
    let genesis_accounts =
        BTreeMap::from([(from1, Account::new(1_000)), (from2, Account::new(500))]);
    let mut cluster = IbftCluster::new_with_execution_engine_factory(
        validators(),
        vec![(from1, 1_000), (from2, 500)],
        || Box::new(ValueTransferEngine),
    );
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx1.clone()));
    cluster.drain(0);
    cluster.submit_request(0, 2, ClientRequest::SubmitTransaction(tx2.clone()));
    cluster.drain(0);

    // Act
    finalize_block(
        &mut cluster,
        vec![tx1, tx2],
        genesis_accounts,
        &ValueTransferEngine,
    );

    // Assert
    let receipts = cluster.node_receipts(0, 0);
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0].status, TransactionStatus::Success);
    assert_eq!(receipts[0].gas_used, INTRINSIC_GAS);
    assert_eq!(receipts[0].cumulative_gas_used, INTRINSIC_GAS);
    assert_eq!(receipts[1].status, TransactionStatus::OutOfGas);
    assert_eq!(receipts[1].gas_used, INTRINSIC_GAS - 1);
    assert_eq!(
        receipts[1].cumulative_gas_used,
        INTRINSIC_GAS + INTRINSIC_GAS - 1
    );
}

#[test]
fn cluster_three_tx_receipt_cumulative_gas_monotonically_increases() {
    // Arrange
    let from1 = [20u8; 20];
    let from2 = [21u8; 20];
    let from3 = [22u8; 20];
    let to = [23u8; 20];
    let tx1 = Transaction::transfer(from1, to, 10, INTRINSIC_GAS, 0);
    let tx2 = Transaction::transfer(from2, to, 20, INTRINSIC_GAS, 0);
    let tx3 = Transaction::transfer(from3, to, 30, INTRINSIC_GAS, 0);
    let genesis_accounts = BTreeMap::from([
        (from1, Account::new(1_000)),
        (from2, Account::new(1_000)),
        (from3, Account::new(1_000)),
    ]);
    let mut cluster = IbftCluster::new_with_execution_engine_factory(
        validators(),
        vec![(from1, 1_000), (from2, 1_000), (from3, 1_000)],
        || Box::new(ValueTransferEngine),
    );
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx1.clone()));
    cluster.drain(0);
    cluster.submit_request(0, 2, ClientRequest::SubmitTransaction(tx2.clone()));
    cluster.drain(0);
    cluster.submit_request(0, 3, ClientRequest::SubmitTransaction(tx3.clone()));
    cluster.drain(0);

    // Act
    finalize_block(
        &mut cluster,
        vec![tx1, tx2, tx3],
        genesis_accounts,
        &ValueTransferEngine,
    );

    // Assert
    let receipts = cluster.node_receipts(0, 0);
    assert_eq!(receipts.len(), 3);
    assert_eq!(receipts[0].status, TransactionStatus::Success);
    assert_eq!(receipts[0].gas_used, INTRINSIC_GAS);
    assert_eq!(receipts[0].cumulative_gas_used, INTRINSIC_GAS);
    assert_eq!(receipts[1].status, TransactionStatus::Success);
    assert_eq!(receipts[1].gas_used, INTRINSIC_GAS);
    assert_eq!(receipts[1].cumulative_gas_used, INTRINSIC_GAS * 2);
    assert_eq!(receipts[2].status, TransactionStatus::Success);
    assert_eq!(receipts[2].gas_used, INTRINSIC_GAS);
    assert_eq!(receipts[2].cumulative_gas_used, INTRINSIC_GAS * 3);
}
