// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::build_block_with_commitments;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_restored_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_with;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::state_root::compute_state_root_with_contract_storage;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::implementations::tiny_evm_engine::TinyEvmEngine;
use std::collections::BTreeMap;

#[test]
fn valid_block_with_correct_commitments_accepted() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(to, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

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
fn valid_block_with_wrong_post_state_root_rejected() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(to, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let mut block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    block.post_state_root[0] ^= 1;
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn valid_block_with_wrong_receipts_root_rejected() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(to, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let mut block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    block.receipts_root[0] ^= 1;
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn valid_block_with_out_of_gas_transaction_correct_roots_accepted() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 20_000, 0);
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(to, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn valid_block_with_out_of_gas_transaction_wrong_roots_rejected() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 20_000, 0);
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(to, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let mut block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    block.receipts_root[0] ^= 1;
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn valid_block_with_contract_storage_mutations_accepted() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let from = [1u8; 20];
    let contract = [2u8; 20];
    let tx = Transaction::new(
        from,
        contract,
        0,
        41_006,
        0,
        vec![0x60, 0x2a, 0x60, 0x00, 0x55, 0xf3],
    );
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(contract, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn valid_block_empty_transactions_accepted() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let mut ctx = setup_context(1, 0);
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let block = build_block_with_commitments(
        0,
        0,
        vec![],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn reexecution_with_locked_block_accepted() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(2, 0);
    ctx.accounts.insert(from, Account::new(1_000));
    ctx.accounts.insert(to, Account::new(0));
    let contract_storage = BTreeMap::new();
    ctx.state_root = compute_state_root_with_contract_storage(&ctx.accounts, &contract_storage);
    let locked_block = build_block_with_commitments(
        0,
        1,
        vec![tx],
        ctx.state_root,
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let block_hash = locked_block.compute_hash();
    let wal = setup_wal_with(|wal| {
        wal.round = 1;
        wal.pending_block = Some(locked_block.clone());
        wal.prepared_certificate = Some(PreparedCertificate {
            height: 0,
            round: 0,
            block_hash,
            signed_prepares: vec![
                (0, SignatureBytes::zeroed()),
                (1, SignatureBytes::zeroed()),
                (2, SignatureBytes::zeroed()),
            ],
        });
    });
    let mut protocol = setup_restored_protocol(wal).with_execution_engine(Box::new(TinyEvmEngine));
    let message = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 0,
        round: 1,
        block: locked_block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &message, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
}
