// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::build_block_with_commitments;
use crate::common::ibft_cluster_test_helpers::finalize_round_with_block;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use barechain_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_PUSH1;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_RETURN;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_SSTORE;
use barechain_etheram_variants::implementations::tiny_evm_gas::INTRINSIC_GAS;
use barechain_etheram_variants::implementations::value_transfer_engine::ValueTransferEngine;
use etheram::common_types::account::Account;
use etheram::common_types::state_root::compute_state_root;
use etheram::common_types::transaction::Transaction;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::timer::timer_event::TimerEvent;
use std::collections::BTreeMap;

#[test]
fn cluster_honest_block_commits_with_reexecution() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    for node in 0..4usize {
        cluster.submit_request(node, 1, ClientRequest::SubmitTransaction(tx.clone()));
        cluster.drain(node);
    }
    let accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&accounts);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &accounts,
        &contract_storage,
        &ValueTransferEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &block);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_height(2), 1);
    assert_eq!(cluster.node_height(3), 1);
}

#[test]
fn cluster_proposer_with_wrong_post_state_root_rejected() {
    // Arrange
    let from = [3u8; 20];
    let to = [4u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    for node in 0..4usize {
        cluster.submit_request(node, 1, ClientRequest::SubmitTransaction(tx.clone()));
        cluster.drain(node);
    }
    let accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&accounts);
    let mut block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &accounts,
        &contract_storage,
        &ValueTransferEngine,
    );
    block.post_state_root[0] ^= 1;

    // Act
    for receiver in 0..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &block));
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(0), 0);
    assert_eq!(cluster.node_height(1), 0);
    assert_eq!(cluster.node_height(2), 0);
    assert_eq!(cluster.node_height(3), 0);
}

#[test]
fn cluster_proposer_with_wrong_receipts_root_rejected() {
    // Arrange
    let from = [5u8; 20];
    let to = [6u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    let accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&accounts);
    let mut block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &accounts,
        &contract_storage,
        &ValueTransferEngine,
    );
    block.receipts_root[0] ^= 1;

    // Act
    for receiver in 0..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &block));
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(0), 0);
    assert_eq!(cluster.node_height(1), 0);
    assert_eq!(cluster.node_height(2), 0);
    assert_eq!(cluster.node_height(3), 0);
}

#[test]
fn cluster_mixed_block_with_gas_failure_commits() {
    // Arrange
    let from1 = [7u8; 20];
    let from2 = [8u8; 20];
    let to = [9u8; 20];
    let tx1 = Transaction::transfer(from1, to, 300, INTRINSIC_GAS, 0);
    let tx2 = Transaction::transfer(from2, to, 50, INTRINSIC_GAS - 1, 0);
    let mut cluster = IbftCluster::new_with_execution_engine_factory(
        validators(),
        vec![(from1, 1_000), (from2, 500)],
        || Box::new(ValueTransferEngine),
    );
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx1.clone()));
    cluster.submit_request(0, 2, ClientRequest::SubmitTransaction(tx2.clone()));
    cluster.drain(0);
    let accounts = BTreeMap::from([(from1, Account::new(1_000)), (from2, Account::new(500))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&accounts);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx1, tx2],
        state_root,
        &accounts,
        &contract_storage,
        &ValueTransferEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &block);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_account(0, from1).map(|a| a.balance), Some(700));
    assert_eq!(cluster.node_account(0, from2).map(|a| a.balance), Some(500));
    assert_eq!(cluster.node_account(0, to).map(|a| a.balance), Some(300));
}

#[test]
fn cluster_contract_execution_block_commits() {
    // Arrange
    let from = [10u8; 20];
    let contract = [11u8; 20];
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
    let accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&accounts);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &accounts,
        &contract_storage,
        &TinyEvmEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &block);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    let slot_zero = [0u8; 32];
    let mut expected_value = [0u8; 32];
    expected_value[31] = 0x2a;
    assert_eq!(
        cluster.node_contract_storage(0, contract, slot_zero),
        Some(expected_value)
    );
}

#[test]
fn cluster_view_change_after_invalid_block() {
    // Arrange
    let from = [12u8; 20];
    let to = [13u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), vec![(from, 1_000)], || {
            Box::new(ValueTransferEngine)
        });
    let accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&accounts);
    let mut invalid_block = build_block_with_commitments(
        0,
        0,
        vec![tx.clone()],
        state_root,
        &accounts,
        &contract_storage,
        &ValueTransferEngine,
    );
    invalid_block.post_state_root[0] ^= 1;

    // Act
    for receiver in 0..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &invalid_block));
    }
    cluster.drain_all();
    for node in 0..4usize {
        cluster.fire_timer(node, TimerEvent::TimeoutRound);
    }
    cluster.drain_all();
    // Assert
    assert_eq!(cluster.node_height(0), 0);
    assert_eq!(cluster.node_height(1), 0);
    assert_eq!(cluster.node_height(2), 0);
    assert_eq!(cluster.node_height(3), 0);
}
