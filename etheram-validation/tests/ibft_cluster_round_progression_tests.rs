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
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use etheram::incoming::timer::timer_event::TimerEvent;

#[test]
fn full_round_four_nodes_all_store_block_at_height_one() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);

    // Act
    finalize_round_after_proposer_timer(&mut cluster, 0, 0, 0, &proposed_block);

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
fn three_of_four_nodes_commit_with_one_offline() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..=2usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
    }
    for replica in 1..=2usize {
        cluster.drain(replica);
    }
    for sender in 0..=2usize {
        for receiver in 0..=2usize {
            cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
        }
    }
    cluster.drain(0);
    cluster.drain(1);
    cluster.drain(2);
    for sender in 0..=2usize {
        for receiver in 0..=2usize {
            cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
        }
    }

    // Act
    cluster.drain(0);
    cluster.drain(1);
    cluster.drain(2);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_height(2), 1);
    assert_eq!(cluster.node_height(3), 0);
}

#[test]
fn two_consecutive_full_rounds_both_blocks_stored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let block_zero = block(0, 0);
    let block_one = block(1, 1);
    finalize_round_after_proposer_timer(&mut cluster, 0, 0, 0, &block_zero);

    // Act
    finalize_round_after_proposer_timer(&mut cluster, 1, 1, 0, &block_one);

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 2);
        assert_eq!(cluster.node_stored_block(node, 0), Some(block_zero.clone()));
        assert_eq!(cluster.node_stored_block(node, 1), Some(block_one.clone()));
    }
}

#[test]
fn hundred_consecutive_full_rounds_all_nodes_converge() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);

    // Act
    for height in 0..100u64 {
        let proposer = (height % 4) as usize;
        let proposed_block = block(height, proposer as u64);
        finalize_round_after_proposer_timer(&mut cluster, proposer, height, 0, &proposed_block);

        for node in 0..4usize {
            assert_eq!(cluster.node_height(node), height + 1);
            assert_eq!(
                cluster.node_stored_block(node, height),
                Some(proposed_block.clone())
            );
        }
    }
}

#[test]
fn five_consecutive_full_rounds_seen_messages_pruning_does_not_block_future_heights() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);

    // Act & Assert
    for height in 0..5u64 {
        let proposer = (height % 4) as usize;
        let proposed_block = block(height, proposer as u64);
        let proposed_block_hash = block_hash(&proposed_block);
        cluster.fire_timer(proposer, TimerEvent::ProposeBlock);
        cluster.drain(proposer);
        for receiver in 0..4usize {
            if receiver != proposer {
                cluster.inject_message(
                    receiver,
                    proposer as u64,
                    pre_prepare(height, 0, &proposed_block),
                );
                cluster.inject_message(
                    receiver,
                    proposer as u64,
                    prepare(height, 0, proposed_block_hash),
                );
            }
        }
        for receiver in 0..4usize {
            if receiver != proposer {
                cluster.drain(receiver);
            }
        }
        for sender in 0..4usize {
            for receiver in 0..4usize {
                if receiver != sender {
                    cluster.inject_message(
                        receiver,
                        sender as u64,
                        prepare(height, 0, proposed_block_hash),
                    );
                }
            }
        }
        cluster.drain_all();
        for sender in 0..4usize {
            for receiver in 0..4usize {
                if receiver != sender {
                    cluster.inject_message(
                        receiver,
                        sender as u64,
                        commit(height, 0, proposed_block_hash),
                    );
                }
            }
        }
        cluster.drain_all();

        for node in 0..4usize {
            assert_eq!(cluster.node_height(node), height + 1);
            assert_eq!(
                cluster.node_stored_block(node, height),
                Some(proposed_block.clone())
            );
        }
    }
}

#[test]
fn empty_block_committed_by_all_nodes() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    assert!(proposed_block.transactions.is_empty());

    // Act
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..4usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for replica in 1..4usize {
        cluster.drain(replica);
    }
    for sender in 1..4usize {
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

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 1);
        let stored = cluster.node_stored_block(node, 0).unwrap();
        assert!(stored.transactions.is_empty());
    }
}
