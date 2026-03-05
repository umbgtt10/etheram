// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::validators;
use etheram_node::common_types::block::Block;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_validation::ibft_cluster::IbftCluster;

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

#[test]
fn duplicate_pre_prepare_flood_from_same_sender_is_ignored_after_first_processing() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    cluster.inject_message(1, 0, pre_prepare(40, 0, 0, &proposed_block));
    cluster.inject_message(1, 0, pre_prepare(40, 0, 0, &proposed_block));
    cluster.inject_message(1, 0, pre_prepare(40, 0, 0, &proposed_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn duplicate_prepare_and_commit_flood_does_not_prematurely_finalize() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(50, 0, 0, proposed_hash));
    cluster.inject_message(0, 1, prepare(50, 0, 0, proposed_hash));
    cluster.inject_message(0, 1, prepare(50, 0, 0, proposed_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(60, 0, 0, proposed_hash));
    cluster.inject_message(0, 1, commit(60, 0, 0, proposed_hash));
    cluster.inject_message(0, 1, commit(60, 0, 0, proposed_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn same_sequence_different_kind_messages_still_follow_normal_consensus_progression() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(70, 0, 0, proposed_hash));
    cluster.inject_message(0, 2, prepare(1, 0, 0, proposed_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(70, 0, 0, proposed_hash));
    cluster.inject_message(0, 2, commit(1, 0, 0, proposed_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_stored_block(0, 0), Some(proposed_block));
}

#[test]
fn duplicate_message_storm_with_true_quorum_still_finalizes() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for _ in 0..20 {
        cluster.inject_message(0, 1, prepare(80, 0, 0, proposed_hash));
    }
    cluster.inject_message(0, 2, prepare(1, 0, 0, proposed_hash));
    cluster.drain(0);
    for _ in 0..20 {
        cluster.inject_message(0, 1, commit(81, 0, 0, proposed_hash));
    }
    cluster.inject_message(0, 2, commit(1, 0, 0, proposed_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_stored_block(0, 0), Some(proposed_block));
}

#[test]
fn duplicate_view_change_single_sender_does_not_reach_new_view_quorum() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    for _ in 0..10 {
        cluster.inject_message(
            1,
            0,
            IbftMessage::ViewChange {
                sequence: 90,
                height: 0,
                round: 1,
                prepared_certificate: None,
            },
        );
    }
    let round_one_block = block(0, 1);
    cluster.inject_message(1, 1, pre_prepare(91, 0, 1, &round_one_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn duplicate_new_view_same_sender_is_ignored_after_first_processing() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let new_view = IbftMessage::NewView {
        sequence: 100,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    };
    cluster.inject_message(2, 1, new_view.clone());
    cluster.inject_message(2, 1, new_view);
    let round_one_block = block(0, 1);
    cluster.inject_message(2, 1, pre_prepare(101, 0, 1, &round_one_block));

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}
