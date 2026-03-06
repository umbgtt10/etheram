// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::build_block_with_commitments;
use crate::common::ibft_cluster_test_helpers::finalize_round_with_block;
use crate::common::ibft_cluster_test_helpers::validators;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::state_root::compute_state_root;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::implementations::value_transfer_engine::ValueTransferEngine;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_validation::ibft_cluster::IbftCluster;
use std::collections::BTreeMap;

#[test]
fn cluster_commits_block_with_descending_gas_price_transactions() {
    // Arrange
    let from_a = [1u8; 20];
    let from_b = [2u8; 20];
    let to = [9u8; 20];
    let tx_high = Transaction::transfer(from_a, to, 1, 21_000, 10, 0);
    let tx_low = Transaction::transfer(from_b, to, 1, 21_000, 5, 0);
    let genesis = vec![(from_a, 1_000), (from_b, 1_000)];
    let mut cluster =
        IbftCluster::new_with_execution_engine_factory(validators(), genesis.clone(), || {
            Box::new(ValueTransferEngine)
        });
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx_high.clone()));
    cluster.submit_request(0, 2, ClientRequest::SubmitTransaction(tx_low.clone()));
    cluster.drain(0);
    let genesis_accounts =
        BTreeMap::from([(from_a, Account::new(1_000)), (from_b, Account::new(1_000))]);
    let contract_storage = BTreeMap::new();
    let state_root = compute_state_root(&genesis_accounts);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx_high, tx_low],
        state_root,
        &genesis_accounts,
        &contract_storage,
        &ValueTransferEngine,
    );

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &block);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
}

#[test]
fn cluster_rejects_pre_prepare_with_ascending_gas_price_transactions() {
    // Arrange
    let from_a = [1u8; 20];
    let from_b = [2u8; 20];
    let to = [9u8; 20];
    let tx_low = Transaction::transfer(from_a, to, 1, 21_000, 5, 0);
    let tx_high = Transaction::transfer(from_b, to, 1, 21_000, 10, 0);
    let genesis = vec![(from_a, 1_000), (from_b, 1_000)];
    let mut cluster = IbftCluster::new_with_execution_engine_factory(validators(), genesis, || {
        Box::new(ValueTransferEngine)
    });
    let bad_block = Block::new(0, 0, vec![tx_low, tx_high], [0u8; 32]);

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &bad_block);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn cluster_rejects_pre_prepare_with_zero_gas_price_transaction() {
    // Arrange
    let from = [1u8; 20];
    let to = [9u8; 20];
    let tx_zero_price = Transaction::transfer(from, to, 1, 21_000, 0, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1_000)]);
    let bad_block = Block::new(0, 0, vec![tx_zero_price], [0u8; 32]);

    // Act
    finalize_round_with_block(&mut cluster, 0, 0, 0, &bad_block);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn submit_zero_gas_price_tx_returns_rejection() {
    // Arrange
    let from = [1u8; 20];
    let to = [9u8; 20];
    let tx = Transaction::transfer(from, to, 1, 21_000, 0, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1_000)]);
    cluster.submit_request(0, 1, ClientRequest::SubmitTransaction(tx));
    cluster.drain(0);

    // Act
    let responses = cluster.drain_client_responses(1);

    // Assert
    assert_eq!(responses.len(), 1);
    assert!(matches!(
        &responses[0],
        ClientResponse::TransactionRejected { .. }
    ));
}
