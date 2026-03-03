// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::validators;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram::common_types::block::Block;
use etheram::incoming::timer::timer_event::TimerEvent;

fn block(height: u64, proposer: u64) -> Block {
    Block::new(height, proposer, vec![], [0u8; 32])
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

trait IbftClusterReplayOps {
    fn try_round_one_commit_on_node_2(&mut self, sequence: u64);
}

impl IbftClusterReplayOps for IbftCluster {
    fn try_round_one_commit_on_node_2(&mut self, sequence: u64) {
        let round_one_block = block(0, 1);
        let round_one_hash = block_hash(&round_one_block);
        self.inject_message(2, 1, pre_prepare(sequence, 0, 1, &round_one_block));
        self.inject_message(2, 0, prepare(1, 0, 1, round_one_hash));
        self.inject_message(2, 1, prepare(1, 0, 1, round_one_hash));
        self.inject_message(2, 3, prepare(1, 0, 1, round_one_hash));
        self.drain(2);
        self.inject_message(2, 0, commit(1, 0, 1, round_one_hash));
        self.inject_message(2, 1, commit(1, 0, 1, round_one_hash));
        self.inject_message(2, 3, commit(1, 0, 1, round_one_hash));
        self.drain(2);
    }
}

#[test]
fn lower_sequence_pre_prepare_after_higher_invalid_pre_prepare_is_rejected() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let invalid_block = block(1, 0);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.inject_message(1, 0, pre_prepare(5, 1, 0, &invalid_block));
    cluster.drain(1);
    cluster.inject_message(1, 0, pre_prepare(4, 0, 0, &valid_block));
    cluster.drain(1);
    cluster.inject_message(1, 0, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 3, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 2, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 3, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn higher_sequence_pre_prepare_after_invalid_pre_prepare_allows_commit_path() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let invalid_block = block(1, 0);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.inject_message(1, 0, pre_prepare(4, 1, 0, &invalid_block));
    cluster.drain(1);
    cluster.inject_message(1, 0, pre_prepare(5, 0, 0, &valid_block));
    cluster.drain(1);
    cluster.inject_message(1, 0, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 3, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 2, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(1, 3, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(valid_block));
}

#[test]
fn high_sequence_from_one_sender_does_not_block_low_sequence_from_other_senders() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(9, 0, 0, [0x77u8; 32]));
    cluster.inject_message(0, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(0);
    cluster.inject_message(0, 2, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_stored_block(0, 0), Some(valid_block));
}

#[test]
fn duplicate_prepare_same_sequence_from_same_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 1, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 1, prepare(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn duplicate_commit_same_sequence_from_same_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn lower_sequence_prepare_after_higher_invalid_from_same_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(5, 0, 0, [0x55u8; 32]));
    cluster.inject_message(0, 1, prepare(4, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn lower_sequence_commit_after_higher_invalid_from_same_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(5, 0, 0, [0x66u8; 32]));
    cluster.inject_message(0, 1, commit(4, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn same_sequence_prepare_and_commit_from_same_sender_are_independent() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(7, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(7, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_stored_block(0, 0), Some(valid_block));
}

#[test]
fn sequence_state_across_height_rejects_old_lower_sender_sequence() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let first_block = block(0, 0);
    let first_hash = block_hash(&first_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(10, 0, 0, first_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, first_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(1, 0, 0, first_hash));
    cluster.inject_message(0, 2, commit(1, 0, 0, first_hash));
    cluster.drain(0);
    let second_block = block(1, 1);
    let second_hash = block_hash(&second_block);
    cluster.inject_message(0, 1, pre_prepare(1, 1, 0, &second_block));
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(9, 1, 0, second_hash));
    cluster.inject_message(0, 2, prepare(1, 1, 0, second_hash));
    cluster.inject_message(0, 3, prepare(1, 1, 0, second_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
}

#[test]
fn higher_sequence_after_height_increment_allows_progress() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let first_block = block(0, 0);
    let first_hash = block_hash(&first_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(10, 0, 0, first_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, first_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(1, 0, 0, first_hash));
    cluster.inject_message(0, 2, commit(1, 0, 0, first_hash));
    cluster.drain(0);
    let second_block = block(1, 1);
    let second_hash = block_hash(&second_block);
    cluster.inject_message(0, 1, pre_prepare(1, 1, 0, &second_block));
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(11, 1, 0, second_hash));
    cluster.inject_message(0, 2, prepare(1, 1, 0, second_hash));
    cluster.inject_message(0, 3, prepare(1, 1, 0, second_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(2, 1, 0, second_hash));
    cluster.inject_message(0, 2, commit(2, 1, 0, second_hash));
    cluster.inject_message(0, 3, commit(2, 1, 0, second_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(second_block));
}

#[test]
fn unknown_sender_high_sequence_does_not_block_validator_low_sequence() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let valid_block = block(0, 0);
    let valid_block_hash = block_hash(&valid_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 99, prepare(100, 0, 0, valid_block_hash));
    cluster.inject_message(0, 1, prepare(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, valid_block_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(1, 0, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, 0, valid_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
}

#[test]
fn view_change_duplicate_same_sequence_from_same_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    cluster.inject_message(
        1,
        0,
        IbftMessage::ViewChange {
            sequence: 5,
            height: 0,
            round: 1,
            prepared_certificate: Some(PreparedCertificate {
                height: 0,
                round: 1,
                block_hash: [0x88u8; 32],
                signed_prepares: vec![
                    (0, SignatureBytes::zeroed()),
                    (0, SignatureBytes::zeroed()),
                    (2, SignatureBytes::zeroed()),
                ],
            }),
        },
    );
    cluster.inject_message(
        1,
        0,
        IbftMessage::ViewChange {
            sequence: 5,
            height: 0,
            round: 1,
            prepared_certificate: None,
        },
    );
    cluster.inject_message(
        1,
        2,
        IbftMessage::ViewChange {
            sequence: 1,
            height: 0,
            round: 1,
            prepared_certificate: None,
        },
    );
    cluster.drain(1);
    cluster.drain(2);

    // Act
    cluster.try_round_one_commit_on_node_2(1);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn new_view_duplicate_same_sequence_from_same_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(
        2,
        1,
        IbftMessage::NewView {
            sequence: 5,
            height: 0,
            round: 1,
            prepared_certificate: None,
            view_change_senders: vec![0, 1],
        },
    );
    cluster.inject_message(
        2,
        1,
        IbftMessage::NewView {
            sequence: 5,
            height: 0,
            round: 1,
            prepared_certificate: None,
            view_change_senders: vec![0, 1, 2],
        },
    );
    cluster.drain(2);

    // Act
    cluster.try_round_one_commit_on_node_2(1);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}
