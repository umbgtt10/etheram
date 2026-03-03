// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::ibft_cluster_test_helpers::block;
use crate::common::ibft_cluster_test_helpers::block_hash;
use crate::common::ibft_cluster_test_helpers::commit;
use crate::common::ibft_cluster_test_helpers::pre_prepare;
use crate::common::ibft_cluster_test_helpers::prepare;
use crate::common::ibft_cluster_test_helpers::validators;
use barechain_core::types::PeerId;
use barechain_etheram_validation::ibft_cluster::IbftCluster;
use barechain_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureScheme;
use etheram::common_types::block::Block;
use etheram::common_types::transaction::Transaction;
use etheram::incoming::timer::timer_event::TimerEvent;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

struct RejectAllSignatureScheme;

impl SignatureScheme for RejectAllSignatureScheme {
    type Signature = SignatureBytes;

    fn sign(&self, _data: &[u8]) -> SignatureBytes {
        SignatureBytes::zeroed()
    }

    fn verify_for_peer(&self, _data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
        false
    }
}

struct ToggleSignatureScheme {
    verify_enabled: Arc<AtomicBool>,
}

impl SignatureScheme for ToggleSignatureScheme {
    type Signature = SignatureBytes;

    fn sign(&self, _data: &[u8]) -> SignatureBytes {
        SignatureBytes::zeroed()
    }

    fn verify_for_peer(&self, _data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
        self.verify_enabled.load(Ordering::SeqCst)
    }
}

#[test]
fn prepare_below_quorum_does_not_commit() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(0, 0, proposed_block_hash));
    cluster.drain(1);
    cluster.inject_message(1, 2, prepare(0, 0, proposed_block_hash));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn prepare_from_non_validator_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &proposed_block));
    cluster.drain(1);
    for _ in 0..3 {
        cluster.inject_message(1, 99, prepare(0, 0, proposed_block_hash));
    }

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn commit_below_quorum_does_not_store_block() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(0, 0, proposed_block_hash));
    cluster.drain(1);
    cluster.inject_message(1, 2, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(1, 3, prepare(0, 0, proposed_block_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(0, 0, proposed_block_hash));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn commit_from_non_validator_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(1, 2, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(1, 3, prepare(0, 0, proposed_block_hash));
    cluster.drain(1);
    for _ in 0..3 {
        cluster.inject_message(1, 99, commit(0, 0, proposed_block_hash));
    }

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_wrong_height_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(1, 1);
    cluster.inject_message(0, 1, pre_prepare(1, 0, &proposed_block));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn pre_prepare_wrong_proposer_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    cluster.inject_message(1, 2, pre_prepare(0, 0, &proposed_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_wrong_round_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    cluster.inject_message(1, 0, pre_prepare(0, 1, &proposed_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn duplicate_prepare_from_single_sender_does_not_reach_quorum() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(0, 1, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(0, 1, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(0, 1, prepare(0, 0, proposed_block_hash));

    // Act
    cluster.drain(0);

    // Assert
    assert_eq!(cluster.node_height(0), 0);
}

#[test]
fn prepare_mixed_valid_and_invalid_votes_requires_true_quorum() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);
    let wrong_block_hash = block_hash(&block(0, 2));
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &proposed_block));
    cluster.inject_message(1, 0, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(1, 2, prepare(0, 0, wrong_block_hash));
    cluster.inject_message(1, 99, prepare(0, 0, proposed_block_hash));
    cluster.drain(1);

    // Act
    cluster.inject_message(1, 2, prepare(0, 0, proposed_block_hash));
    cluster.inject_message(1, 3, prepare(0, 0, proposed_block_hash));
    cluster.drain(1);
    cluster.inject_message(1, 0, commit(0, 0, proposed_block_hash));
    cluster.inject_message(1, 2, commit(0, 0, proposed_block_hash));
    cluster.inject_message(1, 3, commit(0, 0, proposed_block_hash));
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_stored_block(1, 0), Some(proposed_block));
}

#[test]
fn pre_prepare_state_root_mismatch_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let invalid_block = Block::new(0, 0, vec![], [1u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_unknown_transaction_sender_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![]);
    let tx = Transaction::transfer([9u8; 20], [8u8; 20], 1, 21_000, 0);
    let invalid_block = Block::new(0, 0, vec![tx], [0u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_invalid_signature_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new_with_validator_updates_and_signature_scheme(
        validators(),
        vec![],
        vec![],
        |_| Box::new(RejectAllSignatureScheme),
    );
    let proposed_block = block(0, 0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &proposed_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn propose_block_with_invalid_signatures_does_not_reach_commit() {
    // Arrange
    let mut cluster = IbftCluster::new_with_validator_updates_and_signature_scheme(
        validators(),
        vec![],
        vec![],
        |_| Box::new(RejectAllSignatureScheme),
    );
    cluster.fire_timer(0, TimerEvent::ProposeBlock);

    // Act
    cluster.drain_all();

    // Assert
    assert_eq!(cluster.node_height(0), 0);
    assert_eq!(cluster.node_height(1), 0);
    assert_eq!(cluster.node_height(2), 0);
    assert_eq!(cluster.node_height(3), 0);
}

#[test]
fn mixed_signature_cluster_with_one_rejecting_node_still_commits_on_honest_quorum() {
    // Arrange
    let mut cluster = IbftCluster::new_with_validator_updates_and_signature_scheme(
        validators(),
        vec![],
        vec![],
        |peer_id| {
            if peer_id == 3 {
                Box::new(RejectAllSignatureScheme)
            } else {
                Box::new(MockSignatureScheme::new(0))
            }
        },
    );
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);

    // Act
    cluster.fire_timer(0, TimerEvent::ProposeBlock);
    cluster.drain(0);
    for receiver in 1..=2usize {
        cluster.inject_message(receiver, 0, pre_prepare(0, 0, &proposed_block));
        cluster.inject_message(receiver, 0, prepare(0, 0, proposed_block_hash));
    }
    for replica in 1..=2usize {
        cluster.drain(replica);
    }
    for sender in 0..4usize {
        for receiver in 0..=2usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, prepare(0, 0, proposed_block_hash));
            }
        }
    }
    cluster.drain(0);
    cluster.drain(1);
    cluster.drain(2);
    for sender in 0..4usize {
        for receiver in 0..=2usize {
            if receiver != sender {
                cluster.inject_message(receiver, sender as u64, commit(0, 0, proposed_block_hash));
            }
        }
    }
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
fn invalid_then_valid_signature_window_recovers_and_commits() {
    // Arrange
    let verify_enabled = Arc::new(AtomicBool::new(false));
    let mut cluster = IbftCluster::new_with_validator_updates_and_signature_scheme(
        validators(),
        vec![],
        vec![],
        |_| {
            Box::new(ToggleSignatureScheme {
                verify_enabled: Arc::clone(&verify_enabled),
            })
        },
    );
    let invalid_block = block(0, 0);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));
    cluster.drain(1);
    let proposed_block = block(0, 0);
    let proposed_block_hash = block_hash(&proposed_block);

    // Act
    verify_enabled.store(true, Ordering::SeqCst);
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
    assert_eq!(cluster.node_height(0), 1);
    assert_eq!(cluster.node_height(1), 1);
    assert_eq!(cluster.node_height(2), 1);
    assert_eq!(cluster.node_height(3), 1);
}

#[test]
fn pre_prepare_insufficient_balance_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![([1u8; 20], 1)]);
    let tx = Transaction::transfer([1u8; 20], [8u8; 20], 2, 21_000, 0);
    let invalid_block = Block::new(0, 0, vec![tx], [0u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_nonce_mismatch_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![([1u8; 20], 100)]);
    let tx = Transaction::transfer([1u8; 20], [8u8; 20], 1, 21_000, 1);
    let invalid_block = Block::new(0, 0, vec![tx], [0u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_zero_gas_limit_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![([1u8; 20], 100)]);
    let tx = Transaction::transfer([1u8; 20], [8u8; 20], 1, 0, 0);
    let invalid_block = Block::new(0, 0, vec![tx], [0u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}

#[test]
fn pre_prepare_gas_limit_exceeds_max_is_ignored() {
    // Arrange
    let mut cluster = IbftCluster::new(validators(), vec![([1u8; 20], 100)]);
    let tx = Transaction::transfer([1u8; 20], [8u8; 20], 1, 1_000_001, 0);
    let invalid_block = Block::new(0, 0, vec![tx], [0u8; 32]);
    cluster.inject_message(1, 0, pre_prepare(0, 0, &invalid_block));

    // Act
    cluster.drain(1);

    // Assert
    assert_eq!(cluster.node_height(1), 0);
}
