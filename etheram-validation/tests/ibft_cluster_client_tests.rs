// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block;
use crate::common::ibft_cluster_test_helpers::finalize_round_after_proposer_timer;
use crate::common::ibft_cluster_test_helpers::validators;
use etheram::common_types::transaction::Transaction;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::executor::outgoing::external_interface::client_response::TransactionRejectionReason;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram_etheram_validation::ibft_cluster::IbftCluster;

#[test]
fn get_height_at_genesis_returns_height_zero() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.submit_request(0, 99, ClientRequest::GetHeight);

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(99);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], ClientResponse::Height(0));
}

#[test]
fn get_height_after_consensus_returns_incremented_height() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);

    finalize_round_after_proposer_timer(&mut cluster, 0, 0, 0, &proposed_block);
    cluster.submit_request(0, 77, ClientRequest::GetHeight);

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(77);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], ClientResponse::Height(1));
}

#[test]
fn get_balance_nonexistent_account_returns_zero_balance() {
    // Arrange
    let address = [9u8; 20];
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.submit_request(0, 10, ClientRequest::GetBalance(address));

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(10);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0],
        ClientResponse::Balance {
            balance: 0,
            height: 0,
        }
    );
}

#[test]
fn get_balance_genesis_account_returns_balance() {
    // Arrange
    let address = [1u8; 20];
    let mut cluster = IbftCluster::new(validators(), vec![(address, 750)]);
    cluster.submit_request(0, 11, ClientRequest::GetBalance(address));

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(11);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0],
        ClientResponse::Balance {
            balance: 750,
            height: 0,
        }
    );
}

#[test]
fn submit_transaction_valid_tx_returns_accepted() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 12, ClientRequest::SubmitTransaction(tx));

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(12);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], ClientResponse::TransactionAccepted);
}

#[test]
fn submit_transaction_insufficient_balance_returns_rejected() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 500, 21_000, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 100)]);
    cluster.submit_request(0, 13, ClientRequest::SubmitTransaction(tx));

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(13);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0],
        ClientResponse::TransactionRejected {
            reason: TransactionRejectionReason::InsufficientBalance,
        }
    );
}

#[test]
fn submit_transaction_invalid_nonce_returns_rejected() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 5);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 14, ClientRequest::SubmitTransaction(tx));

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(14);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0],
        ClientResponse::TransactionRejected {
            reason: TransactionRejectionReason::InvalidNonce,
        }
    );
}

#[test]
fn submit_transaction_gas_limit_exceeded_returns_rejected() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 1_000_001, 0);
    let mut cluster = IbftCluster::new(validators(), vec![(from, 1000)]);
    cluster.submit_request(0, 15, ClientRequest::SubmitTransaction(tx));

    // Act
    cluster.drain(0);
    let responses = cluster.drain_client_responses(15);

    // Assert
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0],
        ClientResponse::TransactionRejected {
            reason: TransactionRejectionReason::GasLimitExceeded,
        }
    );
}
