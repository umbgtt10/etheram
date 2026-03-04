// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::build_block_with_commitments;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use std::collections::BTreeMap;

#[test]
fn handle_message_pre_prepare_from_proposer_broadcasts_prepare() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
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
fn handle_message_pre_prepare_from_non_proposer_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_already_sent_prepare_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let msg = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: block.clone(),
    });
    protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &msg, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_wrong_height_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let block = Block {
        height: 1,
        proposer: 1,
        transactions: Vec::new(),
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
    };
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 1,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_wrong_round_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 1,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_block_height_mismatch_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block {
        height: 1,
        proposer: 0,
        transactions: Vec::new(),
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
    };
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_locked_block_different_original_proposer_broadcasts_prepare() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let block = Block {
        height: 0,
        proposer: 2,
        transactions: Vec::new(),
        state_root: [0u8; 32],
        post_state_root: [0u8; 32],
        receipts_root: [0u8; 32],
    };
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 0,
                round: 0,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_pre_prepare_state_root_mismatch_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![], [1u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_unknown_transaction_sender_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let tx = Transaction::transfer([7u8; 20], [8u8; 20], 1, 21_000, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![tx], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_insufficient_balance_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(1, 0);
    let from = [1u8; 20];
    ctx.accounts.insert(from, Account::new(1));
    let tx = Transaction::transfer(from, [8u8; 20], 2, 21_000, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![tx], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_nonce_mismatch_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(1, 0);
    let from = [1u8; 20];
    let mut account = Account::new(10);
    account.nonce = 1;
    ctx.accounts.insert(from, account);
    let tx = Transaction::transfer(from, [8u8; 20], 1, 21_000, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![tx], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_zero_gas_limit_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(1, 0);
    let from = [1u8; 20];
    ctx.accounts.insert(from, Account::new(10));
    let tx = Transaction::transfer(from, [8u8; 20], 1, 0, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![tx], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_gas_limit_exceeds_max_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(1, 0);
    let from = [1u8; 20];
    ctx.accounts.insert(from, Account::new(10));
    let tx = Transaction::transfer(from, [8u8; 20], 1, 1_000_001, 0);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![tx], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_pre_prepare_contract_out_of_gas_candidate_still_broadcasts_prepare() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(TinyEvmEngine));
    let mut ctx = setup_context(1, 0);
    let from = [1u8; 20];
    let to = [2u8; 20];
    ctx.accounts.insert(from, Account::new(10));
    let tx = Transaction::new(
        from,
        to,
        0,
        21_001,
        0,
        vec![0x60, 0x2a, 0x60, 0x00, 0x55, 0xf3],
    );
    let contract_storage = BTreeMap::new();
    let block = build_block_with_commitments(
        0,
        0,
        vec![tx],
        [0u8; 32],
        &ctx.accounts,
        &contract_storage,
        &TinyEvmEngine,
    );
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

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
fn handle_message_pre_prepare_block_with_duplicate_nonce_same_sender_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx1 = Transaction::transfer(from, to, 100, 21_000, 0);
    let tx2 = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(1, 0);
    ctx.accounts.insert(
        from,
        Account {
            balance: 1000,
            nonce: 0,
        },
    );
    ctx.accounts.insert(
        to,
        Account {
            balance: 0,
            nonce: 0,
        },
    );
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: Block::new(0, 0, vec![tx1, tx2], [0u8; 32]),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}
