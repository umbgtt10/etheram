// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block;
use crate::common::ibft_cluster_test_helpers::block_hash;
use crate::common::ibft_cluster_test_helpers::commit;
use crate::common::ibft_cluster_test_helpers::finalize_round_after_proposer_timer;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::prepare;
use crate::common::ibft_cluster_test_helpers::validators;
use etheram_node::common_types::block::Block;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_validation::ibft_cluster::IbftCluster;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

fn sequence(height: u64, round: u64, phase: u64) -> u64 {
    (height * 100) + (round * 10) + phase
}

fn view_change(height: u64, round: u64) -> IbftMessage {
    IbftMessage::ViewChange {
        sequence: sequence(height, round, 4),
        height,
        round,
        prepared_certificate: None,
    }
}

fn new_view(height: u64, round: u64, view_change_senders: Vec<u64>) -> IbftMessage {
    IbftMessage::NewView {
        sequence: sequence(height, round, 5),
        height,
        round,
        prepared_certificate: None,
        view_change_senders,
    }
}

fn inject_full_vote_set_for_node(
    cluster: &IbftCluster,
    receiver: usize,
    proposer: u64,
    height: u64,
    round: u64,
    block: &Block,
) {
    let proposed_block_hash = block_hash(block);
    cluster.inject_message(receiver, proposer, pre_prepare(height, round, block));
    cluster.inject_message(
        receiver,
        proposer,
        prepare(height, round, proposed_block_hash),
    );
    for sender in [0u64, 1u64, 3u64] {
        if sender != proposer {
            cluster.inject_message(
                receiver,
                sender,
                prepare(height, round, proposed_block_hash),
            );
        }
    }
    for sender in [0u64, 1u64, 3u64] {
        cluster.inject_message(receiver, sender, commit(height, round, proposed_block_hash));
    }
}

#[test]
fn new_view_round_one_then_full_round_all_nodes_store_block_at_height_one() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    for node in 0..4usize {
        cluster.inject_message(node, 1, new_view(0, 1, vec![0, 1, 2]));
    }
    cluster.drain_all();
    let proposed_block = block(0, 1);

    // Act
    finalize_round_after_proposer_timer(&mut cluster, 1, 0, 1, &proposed_block);

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 1);
        assert_eq!(
            cluster.node_stored_block(node, 0),
            Some(proposed_block.clone())
        );
    }
}

#[test]
fn timeout_round_then_view_change_messages_trigger_new_view_and_round_one_proposal() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    cluster.inject_message(1, 0, view_change(0, 1));
    cluster.inject_message(1, 2, view_change(0, 1));
    let proposed_block = block(0, 1);

    // Act
    cluster.drain(1);
    cluster.inject_message(2, 1, new_view(0, 1, vec![0, 1, 2]));
    cluster.drain(2);
    cluster.inject_message(2, 1, pre_prepare(0, 1, &proposed_block));
    let progressed = cluster.step(2);

    // Assert
    assert!(progressed);
}

#[test]
fn new_view_wrong_proposer_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(2, 0, new_view(0, 1, vec![0, 1, 2]));
    let proposed_block = block(0, 1);
    cluster.inject_message(2, 1, pre_prepare(0, 1, &proposed_block));

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn new_view_below_quorum_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(2, 1, new_view(0, 1, vec![0, 1]));
    let proposed_block = block(0, 1);
    cluster.inject_message(2, 1, pre_prepare(0, 1, &proposed_block));

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn timeout_round_twice_then_round_two_new_view_allows_block_finalization() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    for node in 0..4usize {
        cluster.fire_timer(node, TimerEvent::TimeoutRound);
        cluster.drain(node);
        cluster.fire_timer(node, TimerEvent::TimeoutRound);
        cluster.drain(node);
    }
    for node in 0..4usize {
        cluster.inject_message(node, 2, new_view(0, 2, vec![0, 1, 2]));
    }
    cluster.drain_all();
    let proposed_block = block(0, 2);

    // Act
    finalize_round_after_proposer_timer(&mut cluster, 2, 0, 2, &proposed_block);

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 1);
        assert_eq!(
            cluster.node_stored_block(node, 0),
            Some(proposed_block.clone())
        );
    }
}

#[test]
fn stale_view_change_and_new_view_after_commit_are_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let committed_block = block(0, 0);
    finalize_round_after_proposer_timer(&mut cluster, 0, 0, 0, &committed_block);

    // Act
    cluster.inject_message(2, 1, view_change(0, 1));
    cluster.inject_message(2, 1, new_view(0, 1, vec![0, 1, 2]));
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 1);
    assert_eq!(cluster.node_stored_block(2, 0), Some(committed_block));
}

#[test]
fn new_view_invalid_prepared_certificate_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [3u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (0, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    cluster.inject_message(
        2,
        1,
        IbftMessage::NewView {
            sequence: 0,
            height: 0,
            round: 1,
            prepared_certificate: Some(invalid_prepared_certificate),
            view_change_senders: vec![0, 1, 2],
        },
    );
    let proposed_block = block(0, 1);
    inject_full_vote_set_for_node(&cluster, 2, 1, 0, 1, &proposed_block);

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn new_view_duplicate_view_change_senders_with_unique_quorum_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(2, 1, new_view(0, 1, vec![0, 0, 1, 2]));
    let proposed_block = block(0, 1);
    inject_full_vote_set_for_node(&cluster, 2, 1, 0, 1, &proposed_block);

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn new_view_prepared_certificate_signers_not_subset_of_view_change_senders_accepts_and_commits() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let inconsistent_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [10u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    cluster.inject_message(
        2,
        1,
        IbftMessage::NewView {
            sequence: 0,
            height: 0,
            round: 1,
            prepared_certificate: Some(inconsistent_prepared_certificate),
            view_change_senders: vec![0, 1, 3],
        },
    );
    let proposed_block = block(0, 1);
    inject_full_vote_set_for_node(&cluster, 2, 1, 0, 1, &proposed_block);

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 1);
}

#[test]
fn new_view_without_local_view_change_votes_accepts_and_commits() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(2, 1, new_view(0, 1, vec![0, 1, 3]));
    let proposed_block = block(0, 1);
    inject_full_vote_set_for_node(&cluster, 2, 1, 0, 1, &proposed_block);

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 1);
}

#[test]
fn view_change_wrong_height_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(1, 0, view_change(1, 1));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn view_change_from_non_validator_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(1, 99, view_change(0, 1));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn new_view_wrong_height_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    cluster.inject_message(2, 1, new_view(1, 1, vec![0, 1, 2]));
    let proposed_block = block(0, 1);
    cluster.inject_message(2, 1, pre_prepare(0, 1, &proposed_block));

    // Act
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(2), 0);
}

#[test]
fn drain_all_locked_block_re_proposed_at_round_one_advances_height() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let locked_block = block(0, 0);
    let locked_block_hash = block_hash(&locked_block);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &locked_block));
    cluster.inject_message(1, 0, prepare(0, 0, locked_block_hash));
    cluster.inject_message(1, 2, prepare(0, 0, locked_block_hash));
    cluster.inject_message(1, 3, prepare(0, 0, locked_block_hash));
    cluster.drain(1);
    cluster.fire_timer(1, TimerEvent::TimeoutRound);
    cluster.drain(1);
    for i in [0usize, 2, 3] {
        cluster.inject_message(i, 1, new_view(0, 1, vec![0, 1, 2]));
        cluster.drain(i);
    }

    // Act
    cluster.fire_timer(1, TimerEvent::ProposeBlock);
    cluster.drain(1);
    for i in [0usize, 2, 3] {
        cluster.inject_message(i, 1, pre_prepare(0, 1, &locked_block));
        cluster.inject_message(i, 1, prepare(0, 1, locked_block_hash));
        cluster.drain(i);
    }
    for sender in 0..4u64 {
        for receiver in 0..4usize {
            if receiver != sender as usize {
                cluster.inject_message(receiver, sender, prepare(0, 1, locked_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4u64 {
        for receiver in 0..4usize {
            if receiver != sender as usize {
                cluster.inject_message(receiver, sender, commit(0, 1, locked_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    for i in 0..4 {
        assert_eq!(cluster.node_height(i), 1);
    }
}
