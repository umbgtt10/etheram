// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block;
use crate::common::ibft_cluster_test_helpers::block_hash;
use crate::common::ibft_cluster_test_helpers::commit;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::prepare;
use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use etheram::common_types::transaction::Transaction;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::executor::outgoing::external_interface::client_response::TransactionRejectionReason;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::timer::timer_event::TimerEvent;

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
    let proposed_block_hash = block_hash(&proposed_block);

    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for replica in 1..4usize {
        cluster.drain(replica);
    }
    for sender in 1..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
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
