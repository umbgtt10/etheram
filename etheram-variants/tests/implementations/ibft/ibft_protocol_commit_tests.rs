// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_after_propose_with_tx;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use barechain_core::collection::Collection;
use barechain_core::consensus_protocol::ConsensusProtocol;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram::state::cache::cache_update::CacheUpdate;

#[test]
fn handle_message_commit_quorum_reached_stores_block() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 3);
    assert!(matches!(actions.get(0), Some(Action::ExecuteBlock { .. })));
    assert!(matches!(actions.get(1), Some(Action::StoreBlock { .. })));
    assert!(matches!(actions.get(2), Some(Action::IncrementHeight)));
}

#[test]
fn handle_message_commit_from_unknown_sender_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(99), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_quorum_without_pending_block_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(0, 0);
    let block_hash = [0u8; 32];
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &commit, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_quorum_stores_correct_block() {
    // Arrange
    let mut protocol = setup_protocol();
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block: block.clone(),
    });
    let ctx_replica = setup_context(1, 0);
    protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx_replica);
    let block_hash = block.compute_hash();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(0), &prepare, &ctx_replica);
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx_replica);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx_replica);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(0), &commit, &ctx_replica);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &commit, &ctx_replica);

    // Assert
    assert!(matches!(
        actions.get(0),
        Some(Action::ExecuteBlock { block: stored }) if *stored == block
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::StoreBlock { block: stored }) if *stored == block
    ));
    assert!(matches!(actions.get(2), Some(Action::IncrementHeight)));
}

#[test]
fn handle_message_commit_below_quorum_does_not_store_block() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_wrong_block_hash_does_not_store_block() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let wrong_hash = [0xffu8; 32];
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash: wrong_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &commit, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_wrong_height_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 1,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_wrong_round_returns_empty() {
    // Arrange
    let (mut protocol, ctx, block_hash) = setup_after_propose();
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 1,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

fn reach_commit_quorum_with_tx() -> (Vec<Action<IbftMessage>>, Transaction) {
    let (mut protocol, ctx, block_hash, tx) = setup_after_propose_with_tx();
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);
    let actions = protocol
        .handle_message(&MessageSource::Peer(2), &commit, &ctx)
        .into_inner();
    (actions, tx)
}

#[test]
fn handle_message_commit_quorum_with_tx_emits_remove_pending_for_transaction() {
    // Arrange & Act
    let (actions, tx) = reach_commit_quorum_with_tx();

    // Assert
    assert!(matches!(
        actions.first(),
        Some(Action::UpdateCache {
            update: CacheUpdate::RemovePending(removed),
        }) if removed == &tx
    ));
}

#[test]
fn handle_message_commit_quorum_with_tx_emits_execute_block_action() {
    // Arrange
    let (actions, tx) = reach_commit_quorum_with_tx();

    // Assert
    assert!(matches!(
        actions.get(1),
        Some(Action::ExecuteBlock { block }) if block.transactions.len() == 1 && block.transactions[0] == tx
    ));
}

#[test]
fn handle_commit_two_consecutive_heights_seen_messages_pruned_to_current_height() {
    // Arrange
    let (mut protocol, ctx_0, block_hash_0) = setup_after_propose();
    let prepare_h0 = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash: block_hash_0,
        sender_signature: SignatureBytes::zeroed(),
    });
    let commit_h0 = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash: block_hash_0,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare_h0, &ctx_0);
    protocol.handle_message(&MessageSource::Peer(2), &prepare_h0, &ctx_0);
    protocol.handle_message(&MessageSource::Peer(1), &commit_h0, &ctx_0);
    protocol.handle_message(&MessageSource::Peer(2), &commit_h0, &ctx_0);

    // Act
    let ctx_1 = setup_context(0, 1);
    let block_1 = Block::new(1, 0, vec![], [0u8; 32]);
    let pre_prepare_h1 = Message::Peer(IbftMessage::PrePrepare {
        sequence: 1,
        height: 1,
        round: 0,
        block: block_1,
    });
    let pp_actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare_h1, &ctx_1);
    let block_hash_1 = match pp_actions.get(0) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected Prepare action after PrePrepare at height 1"),
    };
    let prepare_h1 = Message::Peer(IbftMessage::Prepare {
        sequence: 1,
        height: 1,
        round: 0,
        block_hash: block_hash_1,
        sender_signature: SignatureBytes::zeroed(),
    });
    let commit_h1 = Message::Peer(IbftMessage::Commit {
        sequence: 1,
        height: 1,
        round: 0,
        block_hash: block_hash_1,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare_h1, &ctx_1);
    protocol.handle_message(&MessageSource::Peer(2), &prepare_h1, &ctx_1);
    protocol.handle_message(&MessageSource::Peer(1), &commit_h1, &ctx_1);
    protocol.handle_message(&MessageSource::Peer(2), &commit_h1, &ctx_1);

    // Assert
    let wal = protocol.consensus_wal();
    assert!(wal.seen_messages.iter().all(|(h, _, _, _)| *h >= 1));
}

#[test]
fn handle_commit_transaction_with_receiver_near_max_balance_saturates() {
    // Arrange
    let from = [1u8; 20];
    let to = [2u8; 20];
    let tx = Transaction::transfer(from, to, 100, 21_000, 0);
    let mut ctx = setup_context(0, 0);
    ctx.accounts.insert(
        from,
        Account {
            balance: 1_000,
            nonce: 0,
        },
    );
    ctx.accounts.insert(
        to,
        Account {
            balance: u128::MAX - 50,
            nonce: 0,
        },
    );
    ctx.pending_txs.push(tx);
    let mut protocol = setup_protocol();
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let block_hash = match actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected Prepare action"),
    };
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &commit, &ctx);

    // Assert
    assert!(matches!(
        actions.get(0),
        Some(Action::UpdateCache {
            update: CacheUpdate::RemovePending(_),
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::ExecuteBlock { block }) if block.transactions.len() == 1
    ));
}

#[test]
fn handle_message_commit_multi_tx_same_receiver_emits_correct_accumulated_balance() {
    // Arrange
    let sender_a = [1u8; 20];
    let sender_b = [3u8; 20];
    let receiver = [2u8; 20];
    let tx1 = Transaction::transfer(sender_a, receiver, 300, 21_000, 0);
    let tx2 = Transaction::transfer(sender_b, receiver, 200, 21_000, 0);
    let mut ctx = setup_context(0, 0);
    ctx.accounts.insert(
        sender_a,
        Account {
            balance: 1000,
            nonce: 0,
        },
    );
    ctx.accounts.insert(
        sender_b,
        Account {
            balance: 500,
            nonce: 0,
        },
    );
    ctx.accounts.insert(
        receiver,
        Account {
            balance: 0,
            nonce: 0,
        },
    );
    ctx.pending_txs.push(tx1);
    ctx.pending_txs.push(tx2);
    let mut protocol = setup_protocol();
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );
    let block_hash = match actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected Prepare action"),
    };
    let prepare = Message::Peer(IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &prepare, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &prepare, &ctx);
    let commit = Message::Peer(IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    });
    protocol.handle_message(&MessageSource::Peer(1), &commit, &ctx);

    // Act
    let actions = protocol
        .handle_message(&MessageSource::Peer(2), &commit, &ctx)
        .into_inner();

    // Assert
    let remove_pending_count = actions
        .iter()
        .filter(|action| {
            matches!(
                action,
                Action::UpdateCache {
                    update: CacheUpdate::RemovePending(_),
                }
            )
        })
        .count();
    assert_eq!(remove_pending_count, 2);
    assert!(matches!(
        actions.get(2),
        Some(Action::ExecuteBlock { block }) if block.transactions.len() == 2
    ));
}
