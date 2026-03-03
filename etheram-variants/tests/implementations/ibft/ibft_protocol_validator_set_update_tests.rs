// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol_with_validator_updates;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_restored_protocol;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_wal_base;
use barechain_core::collection::Collection;
use barechain_core::consensus_protocol::ConsensusProtocol;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

use barechain_etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use barechain_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use barechain_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use barechain_etheram_variants::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::common_types::block::Block;
use etheram::incoming::timer::timer_event::TimerEvent;
use std::collections::BTreeMap;

trait IbftProtocolValidatorSetUpdateOps {
    fn commit_height(
        &mut self,
        proposer: u64,
        height: u64,
        prepare_senders: [u64; 3],
        commit_senders: [u64; 3],
    );
}

impl IbftProtocolValidatorSetUpdateOps for IbftProtocol {
    fn commit_height(
        &mut self,
        proposer: u64,
        height: u64,
        prepare_senders: [u64; 3],
        commit_senders: [u64; 3],
    ) {
        let propose_context = setup_context(proposer, height);
        let propose_actions = self.handle_message(
            &MessageSource::Timer,
            &Message::Timer(TimerEvent::ProposeBlock),
            &propose_context,
        );
        let proposed_block = match propose_actions.get(0) {
            Some(Action::BroadcastMessage {
                message: IbftMessage::PrePrepare { block, .. },
            }) => block.clone(),
            _ => panic!("expected pre-prepare action"),
        };
        let proposed_block_hash = proposed_block.compute_hash();

        for sender in prepare_senders {
            self.handle_message(
                &MessageSource::Peer(sender),
                &Message::Peer(IbftMessage::Prepare {
                    sequence: (height * 1000) + 100 + sender,
                    height,
                    round: 0,
                    block_hash: proposed_block_hash,
                    sender_signature: SignatureBytes::zeroed(),
                }),
                &propose_context,
            );
        }

        for sender in commit_senders {
            self.handle_message(
                &MessageSource::Peer(sender),
                &Message::Peer(IbftMessage::Commit {
                    sequence: (height * 1000) + 200 + sender,
                    height,
                    round: 0,
                    block_hash: proposed_block_hash,
                    sender_signature: SignatureBytes::zeroed(),
                }),
                &propose_context,
            );
        }
    }
}

#[test]
fn handle_message_timer_propose_block_before_scheduled_update_height_old_proposer_emits_messages() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(2, vec![2, 3, 4, 5])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let old_proposer_context = setup_context(1, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &old_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_after_duplicate_validator_update_old_proposer_emits_messages()
{
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 1, 2, 3])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let old_proposer_context = setup_context(1, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &old_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_after_empty_validator_update_old_proposer_emits_messages() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let old_proposer_context = setup_context(1, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &old_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_after_scheduled_update_new_proposer_emits_messages() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let new_proposer_context = setup_context(2, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &new_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_after_two_updates_second_update_proposer_emits_messages() {
    // Arrange
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
    ];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    protocol.commit_height(2, 1, [1, 3, 4], [1, 3, 4]);
    let second_update_proposer_context = setup_context(4, 2);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &second_update_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_restore_from_wal_after_scheduled_update_keeps_updated_proposer_selection() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let wal = protocol.consensus_wal();
    let mut restored_protocol = setup_restored_protocol(wal);
    let new_proposer_context = setup_context(2, 1);

    // Act
    let actions = restored_protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &new_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_same_target_height_last_update_wins() {
    // Arrange
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(1, vec![2, 3, 4, 5]),
    ];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let stale_proposer_context = setup_context(2, 1);
    let winning_proposer_context = setup_context(3, 1);

    // Act
    let stale_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &stale_proposer_context,
    );
    let winning_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &winning_proposer_context,
    );

    // Assert
    assert_eq!(stale_actions.len(), 0);
    assert_eq!(winning_actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_unsorted_schedule_applies_by_height_order() {
    // Arrange
    let updates = vec![
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
    ];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let first_update_proposer_context = setup_context(2, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &first_update_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timer_propose_block_after_update_to_single_validator_only_single_validator_can_propose(
) {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![9])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let single_validator_context = setup_context(9, 1);
    let non_validator_context = setup_context(2, 1);

    // Act
    let single_validator_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &single_validator_context,
    );
    let non_validator_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &non_validator_context,
    );

    // Assert
    assert_eq!(single_validator_actions.len(), 2);
    assert_eq!(non_validator_actions.len(), 0);
}

#[test]
fn handle_message_prepare_and_commit_after_update_to_seven_validators_recompute_quorum_correctly() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4, 5, 6, 7])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let height_one_context = setup_context(2, 1);
    let propose_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_one_context,
    );
    let block_hash = match propose_actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected prepare action"),
    };

    // Act
    for sender in [1u64, 3, 4, 5] {
        protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Prepare {
                sequence: 300 + sender,
                height: 1,
                round: 0,
                block_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_one_context,
        );
    }
    let before_quorum_actions = protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::Commit {
            sequence: 401,
            height: 1,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &height_one_context,
    );
    let also_before_quorum_actions = protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Commit {
            sequence: 403,
            height: 1,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &height_one_context,
    );
    let at_quorum_actions = protocol.handle_message(
        &MessageSource::Peer(4),
        &Message::Peer(IbftMessage::Commit {
            sequence: 404,
            height: 1,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &height_one_context,
    );

    // Assert
    assert_eq!(before_quorum_actions.len(), 0);
    assert_eq!(also_before_quorum_actions.len(), 0);
    assert_eq!(at_quorum_actions.len(), 0);
    let wal = protocol.consensus_wal();
    assert_eq!(wal.height, 1);
}

#[test]
fn handle_message_prepare_from_removed_validator_after_update_is_ignored() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let height_one_context = setup_context(2, 1);
    let propose_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_one_context,
    );
    let block_hash = match propose_actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected prepare action"),
    };

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 900,
            height: 1,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &height_one_context,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn consensus_wal_after_commit_without_due_update_keeps_original_active_validators() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(2, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);

    // Act
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.active_validators, vec![0, 1, 2, 3]);
    assert!(wal.scheduled_validator_updates.contains_key(&2));
}

#[test]
fn consensus_wal_after_due_update_removes_applied_schedule_entry() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);

    // Act
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.active_validators, vec![1, 2, 3, 4]);
    assert!(!wal.scheduled_validator_updates.contains_key(&1));
}

#[test]
fn handle_message_restore_from_wal_with_future_schedule_applies_only_when_due() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(2, vec![2, 3, 4, 5])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let wal = protocol.consensus_wal();
    let mut restored_protocol = setup_restored_protocol(wal);
    let height_two_context = setup_context(4, 2);

    // Act
    restored_protocol.commit_height(1, 1, [0, 2, 3], [0, 2, 3]);
    let height_two_actions = restored_protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_two_context,
    );

    // Assert
    assert_eq!(height_two_actions.len(), 2);
}

#[test]
fn handle_message_restore_from_wal_after_first_update_preserves_second_update() {
    // Arrange
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
    ];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let wal = protocol.consensus_wal();
    let mut restored_protocol = setup_restored_protocol(wal);
    restored_protocol.commit_height(2, 1, [1, 3, 4], [1, 3, 4]);
    let second_update_proposer_context = setup_context(4, 2);

    // Act
    let actions = restored_protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &second_update_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn handle_message_timeout_round_after_update_uses_updated_round_robin_proposer() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let timeout_context = setup_context(2, 1);
    let round_one_proposer_context = setup_context(3, 1);

    // Act
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &timeout_context,
    );
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &round_one_proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn from_wal_empty_active_validators_falls_back_to_constructor_validators() {
    // Arrange
    let mut wal = setup_wal_base();
    wal.active_validators = vec![];
    wal.scheduled_validator_updates = BTreeMap::new();
    let mut protocol = IbftProtocol::from_wal(
        vec![9, 10, 11, 12],
        Box::new(MockSignatureScheme::new(0)),
        wal,
    );
    let proposer_context = setup_context(10, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &proposer_context,
    );

    // Assert
    assert_eq!(actions.len(), 2);
}

#[test]
fn from_wal_nonempty_active_validators_override_constructor_validators() {
    // Arrange
    let mut wal = setup_wal_base();
    wal.active_validators = vec![1, 2, 3, 4];
    wal.scheduled_validator_updates = BTreeMap::new();
    let mut protocol = IbftProtocol::from_wal(
        vec![9, 10, 11, 12],
        Box::new(MockSignatureScheme::new(0)),
        wal,
    );
    let constructor_set_context = setup_context(10, 1);
    let wal_set_context = setup_context(2, 1);

    // Act
    let constructor_set_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &constructor_set_context,
    );
    let wal_set_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &wal_set_context,
    );

    // Assert
    assert_eq!(constructor_set_actions.len(), 0);
    assert_eq!(wal_set_actions.len(), 2);
}

#[test]
fn handle_message_pre_prepare_from_removed_validator_after_update_is_ignored() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let height_one_context = setup_context(2, 1);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 950,
            height: 1,
            round: 0,
            block: Block::new(1, 0, vec![], [0u8; 32]),
        }),
        &height_one_context,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_commit_from_removed_validator_after_update_is_ignored() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let height_one_context = setup_context(2, 1);
    let propose_actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_one_context,
    );
    let block_hash = match propose_actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected prepare action"),
    };

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Commit {
            sequence: 951,
            height: 1,
            round: 0,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &height_one_context,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn consensus_wal_due_update_clears_view_change_votes() {
    // Arrange
    let mut wal = setup_wal_base();
    wal.view_change_votes.insert((0, 1), vec![0, 1, 2]);
    wal.scheduled_validator_updates.insert(1, vec![1, 2, 3, 4]);
    let mut protocol =
        IbftProtocol::from_wal(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let after_wal = protocol.consensus_wal();

    // Assert
    assert!(after_wal.view_change_votes.is_empty());
}

#[test]
fn view_change_with_stale_cert_three_signers_rejected_after_seven_validator_expansion() {
    // Arrange
    let updates = vec![ValidatorSetUpdate::new(1, vec![0, 1, 2, 3, 4, 5, 6])];
    let mut protocol = setup_protocol_with_validator_updates(updates);
    protocol.commit_height(0, 0, [1, 2, 3], [1, 2, 3]);
    let stale_cert = PreparedCertificate {
        height: 1,
        round: 0,
        block_hash: [0u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let ctx = setup_context(0, 1);
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 500,
        height: 1,
        round: 1,
        prepared_certificate: Some(stale_cert),
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}
