// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::build_block_with_commitments;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use alloc::collections::BTreeMap;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::value_transfer_engine::ValueTransferEngine;

#[test]
fn handle_message_pre_prepare_zero_gas_price_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(1, 0);
    let from = [1u8; 20];
    ctx.accounts.insert(from, Account::new(10));
    let tx = Transaction::transfer(from, [8u8; 20], 1, 21_000, 0, 0);
    let block = Block::new(0, 0, vec![tx], [0u8; 32], BLOCK_GAS_LIMIT);
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_descending_gas_price_broadcasts_prepare() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(ValueTransferEngine));
    let mut ctx = setup_context(1, 0);
    let from_a = [1u8; 20];
    let from_b = [2u8; 20];
    ctx.accounts.insert(from_a, Account::new(1_000));
    ctx.accounts.insert(from_b, Account::new(1_000));
    let tx_high = Transaction::transfer(from_a, [9u8; 20], 1, 21_000, 10, 0);
    let tx_low = Transaction::transfer(from_b, [9u8; 20], 1, 21_000, 5, 0);
    let contract_storage = BTreeMap::new();
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx_high, tx_low],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &ValueTransferEngine,
    );
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}

#[test]
fn handle_message_pre_prepare_ascending_gas_price_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(ValueTransferEngine));
    let mut ctx = setup_context(1, 0);
    let from_a = [1u8; 20];
    let from_b = [2u8; 20];
    ctx.accounts.insert(from_a, Account::new(1_000));
    ctx.accounts.insert(from_b, Account::new(1_000));
    let tx_low = Transaction::transfer(from_a, [9u8; 20], 1, 21_000, 5, 0);
    let tx_high = Transaction::transfer(from_b, [9u8; 20], 1, 21_000, 10, 0);
    let block = Block::new(0, 0, vec![tx_low, tx_high], [0u8; 32], BLOCK_GAS_LIMIT);
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_equal_gas_price_broadcasts_prepare() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(ValueTransferEngine));
    let mut ctx = setup_context(1, 0);
    let from_a = [1u8; 20];
    let from_b = [2u8; 20];
    ctx.accounts.insert(from_a, Account::new(1_000));
    ctx.accounts.insert(from_b, Account::new(1_000));
    let tx_a = Transaction::transfer(from_a, [9u8; 20], 1, 21_000, 5, 0);
    let tx_b = Transaction::transfer(from_b, [9u8; 20], 1, 21_000, 5, 0);
    let contract_storage = BTreeMap::new();
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx_a, tx_b],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &ValueTransferEngine,
    );
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}

#[test]
fn handle_message_pre_prepare_wrong_block_gas_limit_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT - 1);
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_aggregate_tx_gas_exceeds_block_gas_limit_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let tx = Transaction::transfer([1u8; 20], [2u8; 20], 0, BLOCK_GAS_LIMIT + 1, 1, 0);
    let block = Block {
        height: 0,
        proposer: 0,
        transactions: vec![tx],
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
        gas_limit: BLOCK_GAS_LIMIT,
    };
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_equal_gas_wrong_sender_tiebreak_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(ValueTransferEngine));
    let mut ctx = setup_context(1, 0);
    let lower_sender = [1u8; 20];
    let higher_sender = [2u8; 20];
    ctx.accounts.insert(lower_sender, Account::new(1_000));
    ctx.accounts.insert(higher_sender, Account::new(1_000));
    let tx_lower_sender = Transaction::transfer(lower_sender, [9u8; 20], 1, 21_000, 5, 0);
    let tx_higher_sender = Transaction::transfer(higher_sender, [9u8; 20], 1, 21_000, 5, 0);
    let contract_storage = BTreeMap::new();
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx_higher_sender, tx_lower_sender],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &ValueTransferEngine,
    );
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_overflowing_aggregate_gas_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let tx_a = Transaction::transfer([1u8; 20], [2u8; 20], 0, u64::MAX, 1, 0);
    let tx_b = Transaction::transfer([3u8; 20], [4u8; 20], 0, u64::MAX, 1, 0);
    let block = Block {
        height: 0,
        proposer: 0,
        transactions: vec![tx_a, tx_b],
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
        gas_limit: BLOCK_GAS_LIMIT,
    };
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}
