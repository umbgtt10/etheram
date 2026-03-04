// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::validators;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram::common_types::types::{Address, Balance};
use etheram::context::context_dto::Context;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_etheram_validation::ibft_cluster::IbftCluster;
use etheram_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

fn block(height: u64, proposer: u64, state_root: [u8; 32]) -> Block {
    Block::new(height, proposer, vec![], state_root)
}

fn block_hash(block: &Block) -> [u8; 32] {
    block.compute_hash()
}

fn pre_prepare(sequence: u64, height: u64, round: u64, block: &Block) -> IbftMessage {
    IbftMessage::PrePrepare {
        sequence,
        height,
        round,
        block: block.clone(),
    }
}

fn prepare(sequence: u64, height: u64, round: u64, block_hash: [u8; 32]) -> IbftMessage {
    IbftMessage::Prepare {
        sequence,
        height,
        round,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    }
}

fn commit(sequence: u64, height: u64, round: u64, block_hash: [u8; 32]) -> IbftMessage {
    IbftMessage::Commit {
        sequence,
        height,
        round,
        block_hash,
        sender_signature: SignatureBytes::zeroed(),
    }
}

fn view_change(sequence: u64, height: u64, round: u64) -> IbftMessage {
    IbftMessage::ViewChange {
        sequence,
        height,
        round,
        prepared_certificate: None,
    }
}

fn funded_genesis() -> Vec<(Address, Balance)> {
    vec![([1u8; 20], 100)]
}

fn valid_conflicting_blocks(height: u64, proposer: u64) -> (Block, Block) {
    let first = Block::new(
        height,
        proposer,
        vec![Transaction::transfer([1u8; 20], [2u8; 20], 1, 21_000, 0)],
        [0u8; 32],
    );
    let second = Block::new(
        height,
        proposer,
        vec![Transaction::transfer([1u8; 20], [3u8; 20], 2, 21_000, 0)],
        [0u8; 32],
    );
    (first, second)
}

#[test]
fn conflicting_pre_prepare_from_leader_same_height_round_is_ignored_by_follower() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    let (accepted_block, conflicting_block) = valid_conflicting_blocks(0, 0);
    let accepted_hash = block_hash(&accepted_block);
    cluster.inject_message(1, 0, pre_prepare(1, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(2, 0, 0, &conflicting_block));
    cluster.drain(1);

    // Act
    cluster.inject_message(1, 0, prepare(10, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, prepare(10, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, prepare(10, 0, 0, accepted_hash));
    cluster.inject_message(1, 0, commit(20, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, commit(20, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, commit(20, 0, 0, accepted_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn conflicting_pre_prepare_flood_does_not_reach_finalize() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    let (accepted_block, conflicting_block) = valid_conflicting_blocks(0, 0);
    let accepted_hash = block_hash(&accepted_block);
    cluster.inject_message(1, 0, pre_prepare(30, 0, 0, &accepted_block));
    for index in 0..10u64 {
        cluster.inject_message(1, 0, pre_prepare(31 + index, 0, 0, &conflicting_block));
    }
    cluster.drain(1);

    // Act
    cluster.inject_message(1, 0, prepare(60, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, prepare(60, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, prepare(60, 0, 0, accepted_hash));
    cluster.inject_message(1, 0, commit(70, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, commit(70, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, commit(70, 0, 0, accepted_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn honest_single_proposal_with_quorum_still_finalizes_under_byzantine_noise() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0, [0u8; 32]);
    let proposed_hash = block_hash(&proposed_block);
    let wrong_hash = block_hash(&block(0, 0, [9u8; 32]));
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(1, 99, prepare(80, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, prepare(81, 0, 0, wrong_hash));
    cluster.inject_message(1, 2, prepare(81, 0, 0, wrong_hash));
    cluster.inject_message(1, 3, commit(82, 0, 0, wrong_hash));
    cluster.drain(1);

    // Act
    cluster.inject_message(1, 0, pre_prepare(90, 0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(91, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, prepare(91, 0, 0, proposed_hash));
    cluster.inject_message(1, 3, prepare(91, 0, 0, proposed_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(92, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, commit(92, 0, 0, proposed_hash));
    cluster.inject_message(1, 3, commit(92, 0, 0, proposed_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(proposed_block));
}

#[test]
fn malicious_round_then_view_change_allows_next_honest_leader_progress() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let accepted_block = block(0, 0, [0u8; 32]);
    let accepted_hash = block_hash(&accepted_block);
    let conflicting_block = block(0, 0, [7u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(100, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(101, 0, 0, &conflicting_block));
    cluster.inject_message(1, 0, prepare(102, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, prepare(102, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, prepare(102, 0, 0, accepted_hash));
    cluster.drain(1);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    cluster.inject_message(1, 0, view_change(110, 0, 1));
    cluster.inject_message(1, 2, view_change(111, 0, 1));
    cluster.drain(1);
    let round_one_block = block(0, 1, [0u8; 32]);
    let round_one_hash = block_hash(&round_one_block);

    // Act
    cluster.inject_message(1, 1, pre_prepare(120, 0, 1, &round_one_block));
    cluster.inject_message(1, 0, prepare(121, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, prepare(121, 0, 1, round_one_hash));
    cluster.inject_message(1, 3, prepare(121, 0, 1, round_one_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(122, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, commit(122, 0, 1, round_one_hash));
    cluster.inject_message(1, 3, commit(122, 0, 1, round_one_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(round_one_block));
}

#[test]
fn duplicate_conflicting_pre_prepare_same_sequence_is_idempotent() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    let (accepted_block, conflicting_block) = valid_conflicting_blocks(0, 0);
    let accepted_hash = block_hash(&accepted_block);
    cluster.inject_message(1, 0, pre_prepare(230, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(231, 0, 0, &conflicting_block));
    cluster.inject_message(1, 0, pre_prepare(231, 0, 0, &conflicting_block));
    cluster.inject_message(1, 0, pre_prepare(231, 0, 0, &conflicting_block));
    cluster.drain(1);

    // Act
    cluster.inject_message(1, 0, prepare(240, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, prepare(240, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, prepare(240, 0, 0, accepted_hash));
    cluster.inject_message(1, 0, commit(241, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, commit(241, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, commit(241, 0, 0, accepted_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn conflicted_node_stalls_while_other_nodes_finalize() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let accepted_block = block(0, 0, [0u8; 32]);
    let accepted_hash = block_hash(&accepted_block);
    let conflicting_block = block(0, 0, [4u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(250, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(251, 0, 0, &conflicting_block));
    cluster.drain(1);
    for receiver in [0usize, 2, 3] {
        cluster.inject_message(receiver, 0, pre_prepare(252, 0, 0, &accepted_block));
        cluster.inject_message(receiver, 0, prepare(253, 0, 0, accepted_hash));
    }

    // Act
    for receiver in [0usize, 2, 3] {
        cluster.inject_message(receiver, 1, prepare(254, 0, 0, accepted_hash));
        cluster.inject_message(receiver, 2, prepare(254, 0, 0, accepted_hash));
        cluster.inject_message(receiver, 3, prepare(254, 0, 0, accepted_hash));
        cluster.inject_message(receiver, 1, commit(255, 0, 0, accepted_hash));
        cluster.inject_message(receiver, 2, commit(255, 0, 0, accepted_hash));
        cluster.inject_message(receiver, 3, commit(255, 0, 0, accepted_hash));
    }
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(1), 0);
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_height(2), 1);
    assert_eq!(cluster.node_height(3), 1);
}

#[test]
fn conflict_in_round_zero_then_honest_round_one_same_leader_block_finalizes() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let accepted_block = block(0, 0, [0u8; 32]);
    let accepted_hash = block_hash(&accepted_block);
    let conflicting_block = block(0, 0, [3u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(260, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(261, 0, 0, &conflicting_block));
    cluster.drain(1);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    let round_one_block = block(0, 1, [0u8; 32]);
    let round_one_hash = block_hash(&round_one_block);

    // Act
    cluster.inject_message(1, 1, pre_prepare(270, 0, 1, &round_one_block));
    cluster.inject_message(1, 0, prepare(271, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, prepare(271, 0, 1, round_one_hash));
    cluster.inject_message(1, 3, prepare(271, 0, 1, round_one_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(272, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, commit(272, 0, 1, round_one_hash));
    cluster.inject_message(1, 3, commit(272, 0, 1, round_one_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(round_one_block));
    assert_ne!(round_one_hash, accepted_hash);
}

#[test]
fn conflict_recovery_then_next_height_honest_path_still_progresses() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let accepted_block = block(0, 0, [0u8; 32]);
    let conflicting_block = block(0, 0, [2u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(280, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(281, 0, 0, &conflicting_block));
    cluster.drain(1);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    let round_one_block = block(0, 1, [0u8; 32]);
    let round_one_hash = block_hash(&round_one_block);
    cluster.inject_message(1, 1, pre_prepare(290, 0, 1, &round_one_block));
    cluster.inject_message(1, 0, prepare(291, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, prepare(291, 0, 1, round_one_hash));
    cluster.inject_message(1, 3, prepare(291, 0, 1, round_one_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(292, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, commit(292, 0, 1, round_one_hash));
    cluster.inject_message(1, 3, commit(292, 0, 1, round_one_hash));
    cluster.drain(1);
    let height_one_block = block(1, 1, [0u8; 32]);
    let height_one_hash = block_hash(&height_one_block);

    // Act
    cluster.inject_message(1, 1, pre_prepare(300, 1, 0, &height_one_block));
    cluster.inject_message(1, 0, prepare(301, 1, 0, height_one_hash));
    cluster.inject_message(1, 2, prepare(301, 1, 0, height_one_hash));
    cluster.inject_message(1, 3, prepare(301, 1, 0, height_one_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(302, 1, 0, height_one_hash));
    cluster.inject_message(1, 2, commit(302, 1, 0, height_one_hash));
    cluster.inject_message(1, 3, commit(302, 1, 0, height_one_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 2);
    assert_eq!(cluster.node_stored_block(1, 0), Some(round_one_block));
    assert_eq!(cluster.node_stored_block(1, 1), Some(height_one_block));
}

#[test]
fn non_proposer_conflict_noise_then_honest_proposer_block_still_finalizes() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let invalid_a = block(0, 2, [0u8; 32]);
    let invalid_b = block(0, 2, [9u8; 32]);
    cluster.inject_message(1, 2, pre_prepare(310, 0, 0, &invalid_a));
    cluster.inject_message(1, 2, pre_prepare(311, 0, 0, &invalid_b));
    cluster.drain(1);
    let proposed_block = block(0, 0, [0u8; 32]);
    let proposed_hash = block_hash(&proposed_block);

    // Act
    cluster.inject_message(1, 0, pre_prepare(312, 0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(313, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, prepare(313, 0, 0, proposed_hash));
    cluster.inject_message(1, 3, prepare(313, 0, 0, proposed_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(314, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, commit(314, 0, 0, proposed_hash));
    cluster.inject_message(1, 3, commit(314, 0, 0, proposed_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(proposed_block));
}

#[test]
fn stale_height_conflict_noise_then_current_height_honest_block_finalizes() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let stale_a = block(1, 0, [0u8; 32]);
    let stale_b = block(1, 0, [8u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(320, 1, 0, &stale_a));
    cluster.inject_message(1, 0, pre_prepare(321, 1, 0, &stale_b));
    cluster.drain(1);
    let proposed_block = block(0, 0, [0u8; 32]);
    let proposed_hash = block_hash(&proposed_block);

    // Act
    cluster.inject_message(1, 0, pre_prepare(322, 0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(323, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, prepare(323, 0, 0, proposed_hash));
    cluster.inject_message(1, 3, prepare(323, 0, 0, proposed_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(324, 0, 0, proposed_hash));
    cluster.inject_message(1, 2, commit(324, 0, 0, proposed_hash));
    cluster.inject_message(1, 3, commit(324, 0, 0, proposed_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(proposed_block));
}

#[test]
fn malicious_sender_prepare_is_ignored_after_conflict_marking() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    let (accepted_block, conflicting_block) = valid_conflicting_blocks(0, 0);
    let accepted_hash = block_hash(&accepted_block);
    cluster.inject_message(1, 0, pre_prepare(325, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(326, 0, 0, &conflicting_block));
    cluster.drain(1);

    // Act
    cluster.inject_message(1, 0, prepare(327, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, prepare(327, 0, 0, accepted_hash));
    cluster.inject_message(1, 3, prepare(327, 0, 0, accepted_hash));
    cluster.inject_message(1, 0, commit(328, 0, 0, accepted_hash));
    cluster.inject_message(1, 2, commit(328, 0, 0, accepted_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn malicious_sender_cannot_help_round_one_finalize_after_timeout() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    let (accepted_block, conflicting_block) = valid_conflicting_blocks(0, 0);
    cluster.inject_message(1, 0, pre_prepare(333, 0, 0, &accepted_block));
    cluster.inject_message(1, 0, pre_prepare(334, 0, 0, &conflicting_block));
    cluster.drain(1);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    let round_one_block = Block::new(0, 1, vec![], [0u8; 32]);
    cluster.inject_message(1, 1, pre_prepare(335, 0, 1, &round_one_block));
    cluster.drain(1);
    let round_one_hash = block_hash(&round_one_block);

    // Act
    cluster.inject_message(1, 0, prepare(336, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, prepare(336, 0, 1, round_one_hash));
    cluster.inject_message(1, 0, commit(337, 0, 1, round_one_hash));
    cluster.inject_message(1, 2, commit(337, 0, 1, round_one_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn restart_after_invalid_conflict_noise_then_valid_path_progresses() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut protocol = IbftProtocol::new(validators.clone(), Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 330,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [7u8; 32]),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 331,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [9u8; 32]),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(validators, Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 332,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [0u8; 32]),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
}

#[test]
fn malicious_rejection_survives_wal_restart() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut protocol = IbftProtocol::new(validators.clone(), Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    let accepted_block = Block::new(0, 0, vec![], [0u8; 32]);
    let accepted_hash = accepted_block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 130,
            height: 0,
            round: 0,
            block: accepted_block,
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 131,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [8u8; 32]),
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();
    let mut restored =
        IbftProtocol::from_wal(validators, Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = restored.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 132,
            height: 0,
            round: 0,
            block_hash: accepted_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn malicious_sender_view_change_does_not_unlock_round_one_pre_prepare() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    let (accepted_block, conflicting_block) = valid_conflicting_blocks(0, 0);
    cluster.inject_message(2, 0, pre_prepare(340, 0, 0, &accepted_block));
    cluster.inject_message(2, 0, pre_prepare(341, 0, 0, &conflicting_block));
    cluster.drain(2);
    let round_one_block = block(0, 1, [0u8; 32]);

    // Act
    cluster.inject_message(2, 0, view_change(342, 0, 1));
    cluster.inject_message(2, 1, pre_prepare(343, 0, 1, &round_one_block));
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn follower_rejects_oversized_gas_block() {
    // Arrange
    let tx_sender: Address = [1u8; 20];
    let oversized_tx = Transaction::transfer(tx_sender, [2u8; 20], 1, 2_000_000, 0);
    let oversized_block = Block::new(0, 0, vec![oversized_tx], [0u8; 32]);
    let oversized_hash = block_hash(&oversized_block);
    let mut cluster = IbftCluster::new(validators(), funded_genesis());
    cluster.submit_request(
        1,
        1,
        etheram::incoming::external_interface::client_request::ClientRequest::SubmitTransaction(
            Transaction::transfer(tx_sender, [2u8; 20], 1, 21_000, 0),
        ),
    );
    cluster.drain(1);
    cluster.inject_message(1, 0, pre_prepare(350, 0, 0, &oversized_block));

    // Act
    cluster.inject_message(1, 0, prepare(351, 0, 0, oversized_hash));
    cluster.inject_message(1, 2, prepare(352, 0, 0, oversized_hash));
    cluster.inject_message(1, 3, prepare(353, 0, 0, oversized_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(354, 0, 0, oversized_hash));
    cluster.inject_message(1, 2, commit(355, 0, 0, oversized_hash));
    cluster.inject_message(1, 3, commit(356, 0, 0, oversized_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}
