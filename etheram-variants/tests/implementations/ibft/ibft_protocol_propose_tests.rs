// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::state_root::compute_state_root_with_contract_storage;
use etheram::common_types::transaction::Transaction;
use etheram::execution::execution_engine::ExecutionEngine;
use etheram::execution::execution_result::ExecutionResult;
use etheram::execution::receipts_root::compute_receipts_root;
use etheram::execution::transaction_result::TransactionResult;
use etheram::execution::transaction_result::TransactionStatus;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram::state::storage::storage_mutation::StorageMutation;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use std::collections::BTreeMap;

struct FakeExecutionEngine;

impl ExecutionEngine for FakeExecutionEngine {
    fn execute(
        &self,
        _block: &Block,
        _accounts: &BTreeMap<[u8; 20], Account>,
        _contract_storage: &BTreeMap<([u8; 20], [u8; 32]), [u8; 32]>,
    ) -> ExecutionResult {
        ExecutionResult {
            transaction_results: vec![TransactionResult {
                from: [0u8; 20],
                status: TransactionStatus::Success,
                gas_used: 42_000,
                mutations: vec![StorageMutation::UpdateAccount(
                    [1u8; 20],
                    Account {
                        balance: 777,
                        nonce: 1,
                    },
                )],
            }],
        }
    }
}

#[test]
fn handle_message_non_proposer_timer_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_proposer_timer_broadcasts_pre_prepare_and_prepare() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { .. }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}

#[test]
fn handle_message_timer_at_height_one_second_validator_is_proposer() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { .. }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}

#[test]
fn handle_message_timer_proposer_double_fire_returns_empty_second_time() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_proposer_timer_with_engine_sets_post_state_root_and_receipts_root() {
    // Arrange
    let mut protocol = setup_protocol().with_execution_engine(Box::new(FakeExecutionEngine));
    let mut ctx = setup_context(0, 0);
    let tx = Transaction::transfer([1u8; 20], [2u8; 20], 100, 21_000, 0);
    ctx.pending_txs.push(tx);
    ctx.accounts.insert(
        [1u8; 20],
        Account {
            balance: 200,
            nonce: 0,
        },
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    let Some(Action::BroadcastMessage {
        message: IbftMessage::PrePrepare { block, .. },
    }) = actions.get(0)
    else {
        panic!("expected PrePrepare as first action");
    };
    let mut expected_accounts = ctx.accounts.clone();
    expected_accounts.insert(
        [1u8; 20],
        Account {
            balance: 777,
            nonce: 1,
        },
    );
    let expected_post_state_root =
        compute_state_root_with_contract_storage(&expected_accounts, &ctx.contract_storage);
    let expected_receipts_root = compute_receipts_root(&[TransactionResult {
        from: [0u8; 20],
        status: TransactionStatus::Success,
        gas_used: 42_000,
        mutations: vec![],
    }]);
    assert_eq!(block.post_state_root, expected_post_state_root);
    assert_eq!(block.receipts_root, expected_receipts_root);
}

#[test]
fn handle_message_proposer_timer_with_pending_tx_includes_tx_in_pre_prepare_block() {
    // Arrange
    let mut protocol = setup_protocol();
    let mut ctx = setup_context(0, 0);
    let tx = Transaction::transfer([1u8; 20], [2u8; 20], 100, 21_000, 0);
    ctx.pending_txs = vec![tx.clone()];

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    let Some(Action::BroadcastMessage {
        message: IbftMessage::PrePrepare { block, .. },
    }) = actions.get(0)
    else {
        panic!("expected PrePrepare as first action");
    };
    assert_eq!(block.transactions, vec![tx]);
}
