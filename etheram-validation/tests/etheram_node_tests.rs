// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_validation::ibft_test_node::IbftTestNode;
use etheram::common_types::transaction::Transaction;
use etheram::common_types::types::{Address, Hash};
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::executor::outgoing::external_interface::client_response::TransactionRejectionReason;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::timer::timer_event::TimerEvent;

#[test]
fn step_no_events_returns_false() {
    // Arrange
    let mut node = IbftTestNode::new(vec![]);

    // Act & Assert
    assert!(!node.step());
}

#[test]
fn step_propose_block_single_validator_commits_block_increments_height() {
    // Arrange
    let mut node = IbftTestNode::new(vec![]);
    node.fire_timer(TimerEvent::ProposeBlock);

    // Act
    node.step_until_idle();

    // Assert
    assert_eq!(node.node_height(), 1);
}

#[test]
fn step_transaction_submitted_and_committed_updates_sender_balance() {
    // Arrange
    let sender: Address = [1u8; 20];
    let receiver: Address = [2u8; 20];
    let mut node = IbftTestNode::new(vec![(sender, 1_000), (receiver, 0)]);
    node.submit_request(
        1,
        ClientRequest::SubmitTransaction(Transaction::transfer(sender, receiver, 300, 21_000, 0)),
    );
    node.step_until_idle();
    let responses = node.drain_responses(1);
    assert!(matches!(
        responses.first(),
        Some(ClientResponse::TransactionAccepted)
    ));

    // Act
    node.fire_timer(TimerEvent::ProposeBlock);
    node.step_until_idle();

    // Assert
    assert_eq!(node.node_height(), 1);
    assert_eq!(node.node_account(sender).map(|a| a.balance), Some(700));
    assert_eq!(node.node_account(receiver).map(|a| a.balance), Some(300));
}

#[test]
fn step_two_heights_each_with_transaction_final_balances_accumulate_correctly() {
    // Arrange
    let alice: Address = [1u8; 20];
    let bob: Address = [2u8; 20];
    let mut node = IbftTestNode::new(vec![(alice, 1_000), (bob, 0)]);
    node.submit_request(
        1,
        ClientRequest::SubmitTransaction(Transaction::transfer(alice, bob, 100, 21_000, 0)),
    );
    node.step_until_idle();
    node.drain_responses(1);
    node.fire_timer(TimerEvent::ProposeBlock);
    node.step_until_idle();
    assert_eq!(node.node_height(), 1);
    node.submit_request(
        2,
        ClientRequest::SubmitTransaction(Transaction::transfer(bob, alice, 40, 21_000, 0)),
    );
    node.step_until_idle();
    node.drain_responses(2);

    // Act
    node.fire_timer(TimerEvent::ProposeBlock);
    node.step_until_idle();

    // Assert
    assert_eq!(node.node_height(), 2);
    assert_eq!(node.node_account(alice).map(|a| a.balance), Some(940));
    assert_eq!(node.node_account(bob).map(|a| a.balance), Some(60));
}

#[test]
fn state_snapshot_with_genesis_accounts_reports_account_and_contract_storage_counts() {
    // Arrange
    let alice: Address = [1u8; 20];
    let bob: Address = [2u8; 20];
    let node = IbftTestNode::new(vec![(alice, 10), (bob, 20)]);

    // Act
    let account_count = node.snapshot_accounts_count();
    let contract_storage_count = node.snapshot_contract_storage_count();

    // Assert
    assert_eq!(account_count, 2);
    assert_eq!(contract_storage_count, 0);
}

#[test]
fn step_after_committed_transaction_same_nonce_submission_returns_invalid_nonce() {
    // Arrange
    let sender: Address = [1u8; 20];
    let receiver: Address = [2u8; 20];
    let mut node = IbftTestNode::new(vec![(sender, 1_000), (receiver, 0)]);
    let transaction = Transaction::transfer(sender, receiver, 100, 21_000, 0);
    node.submit_request(1, ClientRequest::SubmitTransaction(transaction.clone()));
    node.step_until_idle();
    node.drain_responses(1);
    node.fire_timer(TimerEvent::ProposeBlock);
    node.step_until_idle();

    // Act
    node.submit_request(2, ClientRequest::SubmitTransaction(transaction));
    node.step_until_idle();
    let responses = node.drain_responses(2);

    // Assert
    assert!(matches!(
        responses.first(),
        Some(ClientResponse::TransactionRejected {
            reason: TransactionRejectionReason::InvalidNonce,
        })
    ));
}

#[test]
fn query_contract_storage_without_updates_returns_none() {
    // Arrange
    let node = IbftTestNode::new(vec![]);
    let address: Address = [3u8; 20];
    let slot: Hash = [4u8; 32];

    // Act
    let value = node.node_contract_storage(address, slot);

    // Assert
    assert!(value.is_none());
}
