// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block;
use crate::common::ibft_cluster_test_helpers::commit;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::prepare;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use barechain_etheram_variants::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use etheram::incoming::timer::timer_event::TimerEvent;

trait IbftClusterValidatorUpdateOps {
    fn commit_height_zero_on_node_zero(&mut self);
    fn commit_height_one_on_node_zero_with_proposer_two(&mut self);
    fn finalize_height_zero_on_all_nodes(&mut self);
    fn finalize_height_on_node_zero(
        &mut self,
        height: u64,
        proposer: u64,
        prepare_senders: [u64; 3],
        commit_senders: [u64; 3],
    );
}

impl IbftClusterValidatorUpdateOps for IbftCluster {
    fn commit_height_zero_on_node_zero(&mut self) {
        let proposed_block = block(0, 0);
        let proposed_block_hash = proposed_block.compute_hash();

        self.fire_timer(0, TimerEvent::ProposeBlock);
        self.drain(0);

        for sender in [1u64, 2, 3] {
            self.inject_message(0, sender, prepare(0, 0, proposed_block_hash));
        }
        self.drain(0);

        for sender in [1u64, 2, 3] {
            self.inject_message(0, sender, commit(0, 0, proposed_block_hash));
        }
        self.drain(0);
    }

    fn commit_height_one_on_node_zero_with_proposer_two(&mut self) {
        let proposed_block = block(1, 2);
        let proposed_block_hash = proposed_block.compute_hash();

        self.inject_message(0, 2, pre_prepare(1, 0, &proposed_block));
        self.inject_message(0, 1, prepare(1, 0, proposed_block_hash));
        self.inject_message(0, 3, prepare(1, 0, proposed_block_hash));
        self.inject_message(0, 4, prepare(1, 0, proposed_block_hash));
        self.inject_message(0, 1, commit(1, 0, proposed_block_hash));
        self.inject_message(0, 3, commit(1, 0, proposed_block_hash));
        self.inject_message(0, 4, commit(1, 0, proposed_block_hash));
        self.drain(0);
    }

    fn finalize_height_zero_on_all_nodes(&mut self) {
        let proposed_block = block(0, 0);
        let proposed_block_hash = proposed_block.compute_hash();

        self.fire_timer(0, TimerEvent::ProposeBlock);
        self.drain(0);
        for receiver in 1..4usize {
            self.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
            self.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
        }
        for replica in 1..4usize {
            self.drain(replica);
        }
        for sender in 0..4usize {
            for receiver in 0..4usize {
                if receiver != sender {
                    self.inject_message(
                        receiver,
                        sender as u64,
                        prepare(0, 0, proposed_block_hash),
                    );
                }
            }
        }
        self.drain_all();
        for sender in 0..4usize {
            for receiver in 0..4usize {
                if receiver != sender {
                    self.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
                }
            }
        }
        self.drain_all();
    }

    fn finalize_height_on_node_zero(
        &mut self,
        height: u64,
        proposer: u64,
        prepare_senders: [u64; 3],
        commit_senders: [u64; 3],
    ) {
        let proposed_block = block(height, proposer);
        let proposed_block_hash = proposed_block.compute_hash();

        self.inject_message(0, proposer, pre_prepare(height, 0, &proposed_block));
        for sender in prepare_senders {
            self.inject_message(0, sender, prepare(height, 0, proposed_block_hash));
        }
        for sender in commit_senders {
            self.inject_message(0, sender, commit(height, 0, proposed_block_hash));
        }
        self.drain(0);
    }
}

#[test]
fn inject_message_pre_prepare_old_proposer_before_future_update_height_commits_block() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(2, vec![2, 3, 4, 5])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let old_set_block = block(1, 1);
    let old_set_block_hash = old_set_block.compute_hash();

    // Act
    cluster.inject_message(0, 1, pre_prepare(1, 0, &old_set_block));
    cluster.inject_message(0, 1, prepare(1, 0, old_set_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, old_set_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, old_set_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, old_set_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, old_set_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, old_set_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(old_set_block));
}

#[test]
fn inject_message_pre_prepare_duplicate_validator_update_old_proposer_commits_block() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 1, 2, 3])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let old_set_block = block(1, 1);
    let old_set_block_hash = old_set_block.compute_hash();

    // Act
    cluster.inject_message(0, 1, pre_prepare(1, 0, &old_set_block));
    cluster.inject_message(0, 1, prepare(1, 0, old_set_block_hash));
    cluster.inject_message(0, 2, prepare(1, 0, old_set_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, old_set_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, old_set_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, old_set_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, old_set_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(old_set_block));
}

#[test]
fn inject_message_pre_prepare_old_proposer_after_validator_update_height_does_not_advance() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let invalid_block = block(1, 1);

    // Act
    cluster.inject_message(0, 1, pre_prepare(1, 0, &invalid_block));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert!(cluster.node_stored_block(0, 1).is_none());
}

#[test]
fn inject_message_pre_prepare_new_proposer_after_validator_update_height_commits_block() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let valid_block = block(1, 2);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    cluster.inject_message(0, 2, pre_prepare(1, 0, &valid_block));
    cluster.inject_message(0, 2, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, commit(1, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(valid_block));
}

#[test]
fn inject_message_pre_prepare_second_update_old_proposer_is_rejected_and_new_proposer_commits() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
    ];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    cluster.commit_height_one_on_node_zero_with_proposer_two();
    let invalid_block = block(2, 3);
    let valid_block = block(2, 4);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    cluster.inject_message(0, 3, pre_prepare(2, 0, &invalid_block));
    cluster.inject_message(0, 4, pre_prepare(2, 0, &valid_block));
    cluster.inject_message(0, 2, prepare(2, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(2, 0, valid_block_hash));
    cluster.inject_message(0, 5, prepare(2, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(2, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(2, 0, valid_block_hash));
    cluster.inject_message(0, 5, commit(2, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 3);
    assert_eq!(cluster.node_stored_block(0, 2), Some(valid_block));
}

#[test]
fn inject_message_prepare_from_removed_validator_after_update_does_not_help_reach_quorum() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![0, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let valid_block = block(1, 2);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    cluster.inject_message(0, 2, pre_prepare(1, 0, &valid_block));
    cluster.inject_message(0, 2, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 1, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert!(cluster.node_stored_block(0, 1).is_none());
}

#[test]
fn inject_message_prepare_from_new_validator_after_update_counts_toward_new_quorum() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![0, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let valid_block = block(1, 2);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    cluster.inject_message(0, 2, pre_prepare(1, 0, &valid_block));
    cluster.inject_message(0, 2, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, commit(1, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(valid_block));
}

#[test]
fn inject_message_commit_with_legacy_quorum_after_update_does_not_finalize() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4, 5, 6, 7])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let valid_block = block(1, 2);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    cluster.inject_message(0, 2, pre_prepare(1, 0, &valid_block));
    cluster.inject_message(0, 1, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 5, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, commit(1, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 1);
    assert!(cluster.node_stored_block(0, 1).is_none());
}

#[test]
fn inject_message_commit_reaches_new_quorum_after_update_to_seven_validators_finalizes() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4, 5, 6, 7])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let valid_block = block(1, 2);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    cluster.inject_message(0, 2, pre_prepare(1, 0, &valid_block));
    cluster.inject_message(0, 1, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 5, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 6, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 1, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 5, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 6, commit(1, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(valid_block));
}

#[test]
fn update_boundary_pre_prepare_message_reordering_old_then_new_results_in_single_valid_commit() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let old_block = block(1, 1);
    let new_block = block(1, 2);
    let new_block_hash = new_block.compute_hash();

    // Act
    cluster.inject_message(0, 1, pre_prepare(1, 0, &old_block));
    cluster.inject_message(0, 2, pre_prepare(1, 0, &new_block));
    cluster.inject_message(0, 2, prepare(1, 0, new_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, new_block_hash));
    cluster.inject_message(0, 4, prepare(1, 0, new_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, new_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, new_block_hash));
    cluster.inject_message(0, 4, commit(1, 0, new_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(new_block));
}

#[test]
fn validator_update_full_round_all_nodes_commit_same_block_without_forks() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![0, 1, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.finalize_height_zero_on_all_nodes();
    let block_one = block(1, 1);
    let block_one_hash = block_one.compute_hash();

    // Act
    cluster.fire_timer(1, TimerEvent::ProposeBlock);
    cluster.drain(1);
    for receiver in 0..4usize {
        if receiver != 1 {
            cluster.inject_message(receiver, 1, pre_prepare(1, 0, &block_one));
            cluster.inject_message(receiver, 1, prepare(1, 0, block_one_hash));
        }
    }
    for receiver in [0usize, 2, 3] {
        cluster.drain(receiver);
    }
    for sender in [0u64, 1, 2, 3, 4] {
        for receiver in 0..4usize {
            if receiver as u64 != sender {
                cluster.inject_message(receiver, sender, prepare(1, 0, block_one_hash));
            }
        }
    }
    cluster.drain_all();
    for sender in [0u64, 1, 2, 3, 4] {
        for receiver in 0..4usize {
            if receiver as u64 != sender {
                cluster.inject_message(receiver, sender, commit(1, 0, block_one_hash));
            }
        }
    }
    cluster.drain_all();

    // Assert
    for node in 0..4usize {
        assert_eq!(cluster.node_height(node), 2);
        assert_eq!(cluster.node_stored_block(node, 1), Some(block_one.clone()));
    }
}

#[test]
fn validator_updates_committed_blocks_have_proposers_in_active_set_per_height() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
    ];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    let block_zero = block(0, 0);
    let hash_zero = block_zero.compute_hash();
    let block_one = block(1, 2);
    let hash_one = block_one.compute_hash();
    let block_two = block(2, 4);
    let hash_two = block_two.compute_hash();

    // Act
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for sender in [1u64, 2, 3] {
        cluster.inject_message(0, sender, prepare(0, 0, hash_zero));
    }
    for sender in [1u64, 2, 3] {
        cluster.inject_message(0, sender, commit(0, 0, hash_zero));
    }
    cluster.drain(0);
    cluster.inject_message(0, 2, pre_prepare(1, 0, &block_one));
    for sender in [1u64, 3, 4] {
        cluster.inject_message(0, sender, prepare(1, 0, hash_one));
    }
    for sender in [1u64, 3, 4] {
        cluster.inject_message(0, sender, commit(1, 0, hash_one));
    }
    cluster.drain(0);
    cluster.inject_message(0, 4, pre_prepare(2, 0, &block_two));
    for sender in [2u64, 3, 5] {
        cluster.inject_message(0, sender, prepare(2, 0, hash_two));
    }
    for sender in [2u64, 3, 5] {
        cluster.inject_message(0, sender, commit(2, 0, hash_two));
    }
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 3);
    let stored_zero = cluster.node_stored_block(0, 0);
    let stored_one = cluster.node_stored_block(0, 1);
    let stored_two = cluster.node_stored_block(0, 2);
    assert!(stored_zero
        .as_ref()
        .is_some_and(|block| [0u64, 1, 2, 3].contains(&block.proposer)));
    assert!(stored_one
        .as_ref()
        .is_some_and(|block| [1u64, 2, 3, 4].contains(&block.proposer)));
    assert!(stored_two
        .as_ref()
        .is_some_and(|block| [2u64, 3, 4, 5].contains(&block.proposer)));
}

#[test]
fn validator_update_timeout_view_change_boundary_new_round_proposer_commits_block() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let view_change_block = block(1, 3);
    let view_change_block_hash = view_change_block.compute_hash();

    // Act
    cluster.fire_timer(0, TimerEvent::TimeoutRound);
    cluster.inject_message(
        0,
        1,
        IbftMessage::ViewChange {
            sequence: 600,
            height: 1,
            round: 1,
            prepared_certificate: None,
        },
    );
    cluster.inject_message(
        0,
        2,
        IbftMessage::ViewChange {
            sequence: 601,
            height: 1,
            round: 1,
            prepared_certificate: None,
        },
    );
    cluster.inject_message(
        0,
        4,
        IbftMessage::ViewChange {
            sequence: 602,
            height: 1,
            round: 1,
            prepared_certificate: None,
        },
    );
    cluster.inject_message(
        0,
        3,
        IbftMessage::NewView {
            sequence: 603,
            height: 1,
            round: 1,
            prepared_certificate: None,
            view_change_senders: vec![1, 2, 4],
        },
    );
    cluster.inject_message(
        0,
        3,
        IbftMessage::PrePrepare {
            sequence: 604,
            height: 1,
            round: 1,
            block: view_change_block.clone(),
        },
    );
    for sender in [1u64, 2, 4] {
        cluster.inject_message(
            0,
            sender,
            IbftMessage::Prepare {
                sequence: 610 + sender,
                height: 1,
                round: 1,
                block_hash: view_change_block_hash,
                sender_signature: SignatureBytes::zeroed(),
            },
        );
    }
    for sender in [1u64, 2, 4] {
        cluster.inject_message(
            0,
            sender,
            IbftMessage::Commit {
                sequence: 620 + sender,
                height: 1,
                round: 1,
                block_hash: view_change_block_hash,
                sender_signature: SignatureBytes::zeroed(),
            },
        );
    }
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(view_change_block));
}

#[test]
fn validator_update_old_set_replay_storm_does_not_block_new_set_commit() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    cluster.commit_height_zero_on_node_zero();
    let valid_block = block(1, 2);
    let valid_block_hash = valid_block.compute_hash();

    // Act
    for sequence in 700..730u64 {
        cluster.inject_message(
            0,
            0,
            IbftMessage::PrePrepare {
                sequence,
                height: 1,
                round: 0,
                block: block(1, 0),
            },
        );
        cluster.inject_message(
            0,
            0,
            IbftMessage::Prepare {
                sequence: sequence + 100,
                height: 1,
                round: 0,
                block_hash: block(1, 0).compute_hash(),
                sender_signature: SignatureBytes::zeroed(),
            },
        );
        cluster.inject_message(
            0,
            0,
            IbftMessage::Commit {
                sequence: sequence + 200,
                height: 1,
                round: 0,
                block_hash: block(1, 0).compute_hash(),
                sender_signature: SignatureBytes::zeroed(),
            },
        );
    }
    cluster.inject_message(0, 2, pre_prepare(1, 0, &valid_block));
    cluster.inject_message(0, 2, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, prepare(1, 0, valid_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, valid_block_hash));
    cluster.inject_message(0, 4, commit(1, 0, valid_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
    assert_eq!(cluster.node_stored_block(0, 1), Some(valid_block));
}

#[test]
fn validator_update_long_run_churn_ten_heights_stays_live_and_monotonic() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
        ValidatorSetUpdate::new(3, vec![3, 4, 5, 6]),
        ValidatorSetUpdate::new(4, vec![4, 5, 6, 7]),
        ValidatorSetUpdate::new(5, vec![5, 6, 7, 8]),
        ValidatorSetUpdate::new(6, vec![6, 7, 8, 9]),
        ValidatorSetUpdate::new(7, vec![7, 8, 9, 10]),
        ValidatorSetUpdate::new(8, vec![8, 9, 10, 11]),
        ValidatorSetUpdate::new(9, vec![9, 10, 11, 12]),
    ];
    let mut cluster =
        barechain_etheram_validation::ibft_cluster::IbftCluster::new_with_validator_updates(
            validators,
            vec![],
            updates,
        );
    let proposer_sets = [
        [0u64, 1, 2, 3],
        [1u64, 2, 3, 4],
        [2u64, 3, 4, 5],
        [3u64, 4, 5, 6],
        [4u64, 5, 6, 7],
        [5u64, 6, 7, 8],
        [6u64, 7, 8, 9],
        [7u64, 8, 9, 10],
        [8u64, 9, 10, 11],
        [9u64, 10, 11, 12],
    ];

    // Act
    for height in 0..10u64 {
        let active_set = proposer_sets[height as usize];
        let proposer = active_set[(height as usize) % 4];
        let mut voters = [0u64; 3];
        let mut voter_index = 0usize;
        for validator in active_set {
            if validator != proposer {
                voters[voter_index] = validator;
                voter_index += 1;
            }
        }
        cluster.finalize_height_on_node_zero(height, proposer, voters, voters);
        assert_eq!(cluster.node_height(0), height + 1);
    }

    // Assert
    assert_eq!(cluster.node_height(0), 10);
}

#[test]
fn validator_set_shrink_to_four_nodes_still_reaches_consensus() {
    // Arrange
    let update = ValidatorSetUpdate::new(1, vec![0, 1, 2, 3]);
    let mut cluster =
        IbftCluster::new_with_validator_updates(vec![0, 1, 2, 3, 4], vec![], vec![update]);

    let proposed_block_h0 = block(0, 0);
    let proposed_block_hash_h0 = proposed_block_h0.compute_hash();
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for sender in [1u64, 2, 3, 4] {
        cluster.inject_message(0, sender, prepare(0, 0, proposed_block_hash_h0));
    }
    cluster.drain(0);
    for sender in [1u64, 2, 3, 4] {
        cluster.inject_message(0, sender, commit(0, 0, proposed_block_hash_h0));
    }
    cluster.drain(0);
    assert_eq!(cluster.node_height(0), 1);

    // Act
    let proposed_block = block(1, 1);
    let proposed_block_hash = proposed_block.compute_hash();
    cluster.inject_message(0, 1, pre_prepare(1, 0, &proposed_block));
    cluster.inject_message(0, 2, prepare(1, 0, proposed_block_hash));
    cluster.inject_message(0, 3, prepare(1, 0, proposed_block_hash));
    cluster.drain(0);
    cluster.inject_message(0, 1, commit(1, 0, proposed_block_hash));
    cluster.inject_message(0, 2, commit(1, 0, proposed_block_hash));
    cluster.inject_message(0, 3, commit(1, 0, proposed_block_hash));
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 2);
}
