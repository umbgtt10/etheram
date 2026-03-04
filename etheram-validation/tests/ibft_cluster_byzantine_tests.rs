// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block;
use crate::common::ibft_cluster_test_helpers::block_hash;
use crate::common::ibft_cluster_test_helpers::commit;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::prepare;
use crate::common::ibft_cluster_test_helpers::validators;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram_etheram_validation::ibft_cluster::IbftCluster;
use etheram_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

fn sequence(height: u64, round: u64, phase: u64) -> u64 {
    (height * 100) + (round * 10) + phase
}

fn forged_new_view(
    height: u64,
    round: u64,
    cert: PreparedCertificate,
    view_change_senders: Vec<u64>,
) -> IbftMessage {
    IbftMessage::NewView {
        sequence: sequence(height, round, 5),
        height,
        round,
        prepared_certificate: Some(cert),
        view_change_senders,
    }
}

#[test]
fn byzantine_forged_cert_in_new_view_does_not_block_consensus_mock_limitation() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let forged_block_hash = [0xffu8; 32];
    let forged_cert = PreparedCertificate {
        height: 0,
        round: 0,
        block_hash: forged_block_hash,
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let byzantine_new_view = forged_new_view(0, 1, forged_cert, vec![0, 1, 2]);
    for node in 0..4usize {
        cluster.inject_message(node, 1, byzantine_new_view.clone());
    }
    cluster.drain_all();

    // Act
    let proposed_block = block(0, 1);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(1, TimerEvent::ProposeBlock);
    cluster.drain(1);
    for receiver in [0usize, 2, 3] {
        cluster.inject_message(receiver, 1, pre_prepare(0, 1, &proposed_block));
        cluster.inject_message(receiver, 1, prepare(0, 1, proposed_block_hash));
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 1, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 1, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 1, "node {node} did not advance");
    }
}

#[test]
fn byzantine_commit_for_wrong_block_hash_does_not_prevent_consensus() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    let wrong_hash = [0xAAu8; 32];
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in [1usize, 2, 3] {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for receiver in [1usize, 2, 3] {
        cluster.drain(receiver);
    }
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Act
    for receiver in 0..4usize {
        if receiver != 3 {
            cluster.inject_message(receiver, 3, commit(0, 0, wrong_hash));
        }
    }
    for sender in [0usize, 1, 2] {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 1, "node {node} did not advance");
    }
}

#[test]
fn byzantine_stale_height_messages_after_commit_are_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in [1usize, 2, 3] {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for receiver in [1usize, 2, 3] {
        cluster.drain(receiver);
    }
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Act
    for node in 0..4usize {
        cluster.inject_message(node, 0, pre_prepare(0, 0, &proposed_block));
        for sender in 0..4u64 {
            cluster.inject_message(node, sender, prepare(0, 0, proposed_block_hash));
            cluster.inject_message(node, sender, commit(0, 0, proposed_block_hash));
        }
    }
    cluster.drain_all();

    // Assert
    for node in 0..4usize {
        assert_eq!(
            cluster.node_height(node),
            1,
            "node {node} should remain at height 1"
        );
    }
}

#[test]
fn byzantine_minority_view_change_alone_does_not_trigger_new_view() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let minority_view_change = IbftMessage::ViewChange {
        sequence: sequence(0, 1, 4),
        height: 0,
        round: 1,
        prepared_certificate: None,
    };
    for receiver in 0..4usize {
        cluster.inject_message(receiver, 3, minority_view_change.clone());
    }
    cluster.drain_all();

    // Act
    let proposed_block = block(0, 1);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(1, TimerEvent::ProposeBlock);
    cluster.drain(1);
    for receiver in [0usize, 2, 3] {
        cluster.inject_message(receiver, 1, pre_prepare(0, 1, &proposed_block));
        cluster.inject_message(receiver, 1, prepare(0, 1, proposed_block_hash));
    }
    for receiver in [0usize, 2, 3] {
        cluster.drain(receiver);
    }
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 1, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in 0..4usize {
        for receiver in 0..4usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 1, proposed_block_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 1, "node {node} did not advance");
    }
}
