// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::build_block_with_commitments;
use crate::common::ibft_cluster_test_helpers::finalize_round_with_block;
use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use barechain_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram::common_types::account::Account;
use etheram::common_types::state_root::compute_state_root;
use etheram::common_types::transaction::Transaction;
use etheram::incoming::external_interface::client_request::ClientRequest;
use std::collections::BTreeMap;

#[test]
fn replicas_commit_block_containing_transaction() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&genesis_accounts);
    let proposed_block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &genesis_accounts,
        &contract_storage,
        &TinyEvmEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &proposed_block);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    let stored = cluster.node_stored_block(0, 0).unwrap();
    assert_eq!(stored.height, proposed_block.height);
    assert_eq!(stored.proposer, proposed_block.proposer);
    assert_eq!(stored.transactions, proposed_block.transactions);
    assert_eq!(stored.state_root, proposed_block.state_root);
}

#[test]
fn committed_transaction_transfers_balance_between_accounts() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx.clone()));
    cluster.drain(0);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&genesis_accounts);
    let proposed_block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        state_root,
        &genesis_accounts,
        &contract_storage,
        &TinyEvmEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &proposed_block);

    // Assert
    assert_eq!(cluster.node_account(0, from).map(|a| a.balance), Some(900));
    assert_eq!(cluster.node_account(0, to).map(|a| a.balance), Some(100));
}

#[test]
fn two_consecutive_heights_each_with_transaction_final_balances_are_correct() {
    // Arrange
    let from: [u8; 20] = [1u8; 20];
    let to: [u8; 20] = [2u8; 20];
    let tx_zero = Transaction::transfer(from, to, 100, 21_000, 0);
    let tx_one = Transaction::transfer(to, from, 50, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1_000)]);
    for node in 0..4usize {
        cluster.submit_request(node, 1, ClientRequest::SubmitTransaction(tx_zero.clone()));
        cluster.drain(node);
    }
    let genesis_accounts = BTreeMap::from([(from, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root_genesis = compute_state_root(&genesis_accounts);
    let block_zero = build_block_with_commitments(
        0,
        0,
        vec![tx_zero],
        state_root_genesis,
        &genesis_accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    finalize_round_with_block(&mut cluster, 0, 0, 0, &block_zero);
    assert_eq!(cluster.node_height(0), 1);
    let accounts_after_zero = BTreeMap::from([
        (
            from,
            Account {
                balance: 900,
                nonce: 1,
            },
        ),
        (to, Account::new(100)),
    ]);
    let state_root_one = compute_state_root(&accounts_after_zero);
    for node in 0..4usize {
        cluster.submit_request(node, 2, ClientRequest::SubmitTransaction(tx_one.clone()));
        cluster.drain(node);
    }
    let block_one = build_block_with_commitments(
        1,
        1,
        vec![tx_one],
        state_root_one,
        &accounts_after_zero,
        &contract_storage,
        &TinyEvmEngine,
    );
    finalize_round_with_block(&mut cluster, 1, 1, 0, &block_one);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_account(0, from).map(|a| a.balance), Some(950));
    assert_eq!(cluster.node_account(0, to).map(|a| a.balance), Some(50));
}

#[test]
fn submit_transaction_after_propose_fires_tx_included_in_next_block() {
    // Arrange
    let from = [10u8; 20];
    let to = [11u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    let genesis_accounts = BTreeMap::from([(from, Account::new(1000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&genesis_accounts);
    let empty_block = build_block_with_commitments(
        0,
        0,
        vec![],
        state_root,
        &genesis_accounts,
        &contract_storage,
        &TinyEvmEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &empty_block);
    let accounts_after_empty = BTreeMap::from([(from, Account::new(1000))]);
    let tx_block = build_block_with_commitments(
        1,
        1,
        vec![tx],
        state_root,
        &accounts_after_empty,
        &contract_storage,
        &TinyEvmEngine,
    );
    for node in 0..4usize {
        cluster.submit_request(
            node,
            1,
            ClientRequest::SubmitTransaction(tx_block.transactions[0].clone()),
        );
        cluster.drain(node);
    }
    finalize_round_with_block(&mut cluster, 1, 1, 0, &tx_block);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    let committed_block = cluster.node_stored_block(0, 0).unwrap();
    assert!(committed_block.transactions.is_empty());
    assert_eq!(cluster.node_account(0, from).map(|a| a.balance), Some(900));
}
