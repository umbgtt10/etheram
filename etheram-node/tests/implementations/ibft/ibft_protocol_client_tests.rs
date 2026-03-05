// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::state::cache::cache_update::CacheUpdate;

#[test]
fn handle_message_get_height_client_request_returns_height_response() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 3);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(42),
        &Message::Client(ClientRequest::GetHeight),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::SendClientResponse {
            client_id: 42,
            response: ClientResponse::Height(3),
        })
    ));
}

#[test]
fn handle_message_non_client_source_with_client_message_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 5);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Client(ClientRequest::GetHeight),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_get_balance_existing_account_returns_balance() {
    // Arrange
    let mut protocol = setup_protocol();
    let address = [1u8; 20];
    let mut ctx = setup_context(0, 7);
    ctx.accounts.insert(address, Account::new(500));

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(20),
        &Message::Client(ClientRequest::GetBalance(address)),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::SendClientResponse {
            client_id: 20,
            response: ClientResponse::Balance {
                balance: 500,
                height: 7,
            },
        })
    ));
}

#[test]
fn handle_message_get_balance_unknown_address_returns_zero_balance() {
    // Arrange
    let mut protocol = setup_protocol();
    let address = [9u8; 20];
    let ctx = setup_context(0, 2);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(30),
        &Message::Client(ClientRequest::GetBalance(address)),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::SendClientResponse {
            client_id: 30,
            response: ClientResponse::Balance {
                balance: 0,
                height: 2,
            },
        })
    ));
}

#[test]
fn handle_message_submit_transaction_valid_tx_returns_accepted() {
    // Arrange
    let mut protocol = setup_protocol();
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(0, 0);
    ctx.accounts.insert(from, Account::new(500));

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(50),
        &Message::Client(ClientRequest::SubmitTransaction(tx.clone())),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::UpdateCache {
            update: CacheUpdate::AddPending(_),
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::SendClientResponse {
            client_id: 50,
            response: ClientResponse::TransactionAccepted,
        })
    ));
}

#[test]
fn handle_message_submit_transaction_insufficient_balance_returns_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 200, 21_000, 0);
    let mut ctx = setup_context(0, 0);
    ctx.accounts.insert(from, Account::new(50));

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(60),
        &Message::Client(ClientRequest::SubmitTransaction(tx)),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::SendClientResponse {
            client_id: 60,
            response: ClientResponse::TransactionRejected {
                reason: TransactionRejectionReason::InsufficientBalance,
            },
        })
    ));
}

#[test]
fn handle_message_submit_transaction_invalid_nonce_returns_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 5);
    let mut ctx = setup_context(0, 0);
    ctx.accounts.insert(from, Account::new(1000));

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(70),
        &Message::Client(ClientRequest::SubmitTransaction(tx)),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::SendClientResponse {
            client_id: 70,
            response: ClientResponse::TransactionRejected {
                reason: TransactionRejectionReason::InvalidNonce,
            },
        })
    ));
}

#[test]
fn handle_message_submit_transaction_gas_limit_exceeded_returns_rejected() {
    // Arrange
    let mut protocol = setup_protocol();
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 1_000_001, 0);
    let mut ctx = setup_context(0, 0);
    ctx.accounts.insert(from, Account::new(1000));

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Client(80),
        &Message::Client(ClientRequest::SubmitTransaction(tx)),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::SendClientResponse {
            client_id: 80,
            response: ClientResponse::TransactionRejected {
                reason: TransactionRejectionReason::GasLimitExceeded,
            },
        })
    ));
}
