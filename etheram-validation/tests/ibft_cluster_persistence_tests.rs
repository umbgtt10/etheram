// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::context::context_dto::Context;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_node::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use std::collections::VecDeque;

struct MiniNode {
    peer_id: u64,
    protocol: IbftProtocol,
    height: u64,
    stored_block: Option<Block>,
}

impl MiniNode {
    fn new(peer_id: u64, validators: Vec<u64>) -> Self {
        Self {
            peer_id,
            protocol: IbftProtocol::new(validators, Box::new(MockSignatureScheme::new(0))),
            height: 0,
            stored_block: None,
        }
    }

    fn context(&self) -> Context {
        Context::new(self.peer_id, self.height, [0u8; 32])
    }

    fn new_with_validator_updates(
        peer_id: u64,
        validators: Vec<u64>,
        updates: Vec<ValidatorSetUpdate>,
    ) -> Self {
        Self {
            peer_id,
            protocol: IbftProtocol::new_with_validator_updates(
                validators,
                Box::new(MockSignatureScheme::new(0)),
                updates,
            ),
            height: 0,
            stored_block: None,
        }
    }

    fn apply_local_mutations(
        &mut self,
        message_actions: &impl Collection<Item = Action<IbftMessage>>,
    ) {
        for index in 0..message_actions.len() {
            match message_actions.get(index) {
                Some(Action::StoreBlock { block }) => {
                    self.stored_block = Some(block.clone());
                }
                Some(Action::IncrementHeight) => {
                    self.height += 1;
                }
                _ => {}
            }
        }
    }
}

fn queue_broadcasts(
    from_peer: u64,
    actions: &etheram_core::node_common::action_collection::ActionCollection<Action<IbftMessage>>,
    validators: &[u64],
) -> VecDeque<(u64, u64, IbftMessage)> {
    let mut queue = VecDeque::new();
    for index in 0..actions.len() {
        if let Some(Action::BroadcastMessage { message }) = actions.get(index) {
            for peer in validators {
                if *peer != from_peer {
                    queue.push_back((from_peer, *peer, message.clone()));
                }
            }
        }
    }
    queue
}

fn find_index(nodes: &[MiniNode], peer_id: u64) -> usize {
    nodes
        .iter()
        .position(|node| node.peer_id == peer_id)
        .unwrap()
}

#[test]
fn restart_mid_prepare_then_recover_wal_all_nodes_finalize_same_block() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut nodes = vec![
        MiniNode::new(0, validators.clone()),
        MiniNode::new(1, validators.clone()),
        MiniNode::new(2, validators.clone()),
        MiniNode::new(3, validators.clone()),
    ];
    let mut queue = VecDeque::new();
    let proposer_ctx = nodes[0].context();
    let proposer_actions = nodes[0].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &proposer_ctx,
    );
    queue.extend(queue_broadcasts(0, &proposer_actions, &validators));
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }
    let wal = nodes[2].protocol.consensus_wal();
    nodes[2].protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        wal,
    );
    let block = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let block_hash = block.compute_hash();
    for from in 0..4u64 {
        for to in 0..4u64 {
            if from != to {
                queue.push_back((
                    from,
                    to,
                    IbftMessage::Commit {
                        sequence: 99,
                        height: 0,
                        round: 0,
                        block_hash,
                        sender_signature: SignatureBytes::zeroed(),
                    },
                ));
            }
        }
    }

    // Act
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
    }

    // Assert
    for node in &nodes {
        assert_eq!(node.height, 1);
        assert_eq!(node.stored_block, Some(block.clone()));
    }
}

#[test]
fn restart_after_timeout_then_view_change_reaches_new_view_and_round_one_proposal() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut node = MiniNode::new(1, validators.clone());
    let ctx = node.context();
    node.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let wal = node.protocol.consensus_wal();
    node.protocol = IbftProtocol::from_wal(validators, Box::new(MockSignatureScheme::new(0)), wal);
    let first_vote = Message::Peer(IbftMessage::ViewChange {
        sequence: 30,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    let second_vote = Message::Peer(IbftMessage::ViewChange {
        sequence: 31,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });

    // Act
    node.protocol
        .handle_message(&MessageSource::Peer(0), &first_vote, &ctx);
    let actions = node
        .protocol
        .handle_message(&MessageSource::Peer(2), &second_vote, &ctx);
    let propose_actions = node.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView { round: 1, .. }
        })
    ));
    assert_eq!(propose_actions.len(), 2);
    assert!(matches!(
        propose_actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { round: 1, .. }
        })
    ));
}

#[test]
fn restart_preserves_replay_state_and_rejects_stale_duplicate_messages() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut node = MiniNode::new(1, validators.clone());
    let ctx = node.context();
    let block = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    node.protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 8,
            height: 0,
            round: 0,
            block,
        }),
        &ctx,
    );
    let wal = node.protocol.consensus_wal();
    node.protocol = IbftProtocol::from_wal(validators, Box::new(MockSignatureScheme::new(0)), wal);

    // Act
    let actions = node.protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 8,
            height: 0,
            round: 0,
            block: Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT),
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn staggered_multi_node_restarts_mid_height_still_converge_on_same_block() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut nodes = vec![
        MiniNode::new(0, validators.clone()),
        MiniNode::new(1, validators.clone()),
        MiniNode::new(2, validators.clone()),
        MiniNode::new(3, validators.clone()),
    ];
    let mut queue = VecDeque::new();
    let proposer_ctx = nodes[0].context();
    let proposer_actions = nodes[0].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &proposer_ctx,
    );
    queue.extend(queue_broadcasts(0, &proposer_actions, &validators));
    for _ in 0..4usize {
        if let Some((from, to, message)) = queue.pop_front() {
            let index = find_index(&nodes, to);
            let ctx = nodes[index].context();
            let actions = nodes[index].protocol.handle_message(
                &MessageSource::Peer(from),
                &Message::Peer(message),
                &ctx,
            );
            nodes[index].apply_local_mutations(&actions);
            queue.extend(queue_broadcasts(to, &actions, &validators));
        }
    }
    let wal_node_2 = nodes[2].protocol.consensus_wal();
    nodes[2].protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        wal_node_2,
    );
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }
    let wal_node_3 = nodes[3].protocol.consensus_wal();
    nodes[3].protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        wal_node_3,
    );
    let block = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let block_hash = block.compute_hash();
    for from in 0..4u64 {
        for to in 0..4u64 {
            if from != to {
                queue.push_back((
                    from,
                    to,
                    IbftMessage::Commit {
                        sequence: 120,
                        height: 0,
                        round: 0,
                        block_hash,
                        sender_signature: SignatureBytes::zeroed(),
                    },
                ));
            }
        }
    }

    // Act
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
    }

    // Assert
    for node in &nodes {
        assert_eq!(node.height, 1);
        assert_eq!(node.stored_block, Some(block.clone()));
    }
}

#[test]
fn restart_after_commit_then_next_height_proposer_still_advances_consensus() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut nodes = vec![
        MiniNode::new(0, validators.clone()),
        MiniNode::new(1, validators.clone()),
        MiniNode::new(2, validators.clone()),
        MiniNode::new(3, validators.clone()),
    ];
    let mut queue = VecDeque::new();
    let height_zero_ctx = nodes[0].context();
    let height_zero_actions = nodes[0].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_zero_ctx,
    );
    queue.extend(queue_broadcasts(0, &height_zero_actions, &validators));
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }
    let block_zero = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let hash_zero = block_zero.compute_hash();
    for from in 0..4u64 {
        for to in 0..4u64 {
            if from != to {
                queue.push_back((
                    from,
                    to,
                    IbftMessage::Commit {
                        sequence: 121,
                        height: 0,
                        round: 0,
                        block_hash: hash_zero,
                        sender_signature: SignatureBytes::zeroed(),
                    },
                ));
            }
        }
    }
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
    }
    let wal = nodes[1].protocol.consensus_wal();
    nodes[1].protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        wal,
    );
    let next_height_ctx = nodes[1].context();

    // Act
    let next_height_actions = nodes[1].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &next_height_ctx,
    );

    // Assert
    assert_eq!(nodes[1].height, 1);
    assert_eq!(next_height_actions.len(), 2);
    assert!(matches!(
        next_height_actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare {
                height: 1,
                round: 0,
                ..
            }
        })
    ));
}

#[test]
fn multi_restart_during_view_change_keeps_liveness_and_emits_single_new_view() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut node_one = MiniNode::new(1, validators.clone());
    let mut node_two = MiniNode::new(2, validators.clone());
    node_one.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &node_one.context(),
    );
    node_two.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &node_two.context(),
    );
    let wal_one = node_one.protocol.consensus_wal();
    let wal_two = node_two.protocol.consensus_wal();
    node_one.protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        wal_one,
    );
    node_two.protocol =
        IbftProtocol::from_wal(validators, Box::new(MockSignatureScheme::new(0)), wal_two);

    // Act
    let first = node_one.protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 41,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &node_one.context(),
    );
    let second = node_one.protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 42,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &node_one.context(),
    );
    let third = node_one.protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::ViewChange {
            sequence: 43,
            height: 0,
            round: 1,
            prepared_certificate: None,
        }),
        &node_one.context(),
    );
    let propose = node_one.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &node_one.context(),
    );
    let new_view_count = [first, second, third]
        .iter()
        .flat_map(|actions| (0..actions.len()).map(|index| actions.get(index)))
        .filter(|action| {
            matches!(
                action,
                Some(Action::BroadcastMessage {
                    message: IbftMessage::NewView { .. }
                })
            )
        })
        .count();

    // Assert
    assert_eq!(new_view_count, 1);
    assert_eq!(propose.len(), 2);
    assert!(matches!(
        propose.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { round: 1, .. }
        })
    ));
}

#[test]
fn restart_at_update_boundary_old_proposer_stays_rejected_new_proposer_finalizes() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![ValidatorSetUpdate::new(1, vec![1, 2, 3, 4])];
    let mut node = MiniNode::new_with_validator_updates(0, validators.clone(), updates);
    let height_zero_ctx = node.context();
    let height_zero_actions = node.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_zero_ctx,
    );
    let height_zero_hash = match height_zero_actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected prepare action"),
    };
    for sender in [1u64, 2, 3] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Prepare {
                sequence: 150 + sender,
                height: 0,
                round: 0,
                block_hash: height_zero_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_zero_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    for sender in [1u64, 2, 3] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Commit {
                sequence: 250 + sender,
                height: 0,
                round: 0,
                block_hash: height_zero_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_zero_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    let wal = node.protocol.consensus_wal();
    node.protocol = IbftProtocol::from_wal(validators, Box::new(MockSignatureScheme::new(0)), wal);
    let height_one_ctx = node.context();
    let old_block = Block::new(1, 1, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let new_block = Block::new(1, 2, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let new_block_hash = new_block.compute_hash();

    // Act
    let old_actions = node.protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 300,
            height: 1,
            round: 0,
            block: old_block,
        }),
        &height_one_ctx,
    );
    let new_pre_prepare_actions = node.protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 301,
            height: 1,
            round: 0,
            block: new_block.clone(),
        }),
        &height_one_ctx,
    );
    node.apply_local_mutations(&new_pre_prepare_actions);
    for sender in [2u64, 3, 4] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Prepare {
                sequence: 310 + sender,
                height: 1,
                round: 0,
                block_hash: new_block_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_one_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    for sender in [2u64, 3, 4] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Commit {
                sequence: 320 + sender,
                height: 1,
                round: 0,
                block_hash: new_block_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_one_ctx,
        );
        node.apply_local_mutations(&actions);
    }

    // Assert
    assert_eq!(old_actions.len(), 0);
    assert_eq!(node.height, 2);
    assert_eq!(node.stored_block, Some(new_block));
}

#[test]
fn restart_after_first_update_second_update_still_applies_and_finalizes() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let updates = vec![
        ValidatorSetUpdate::new(1, vec![1, 2, 3, 4]),
        ValidatorSetUpdate::new(2, vec![2, 3, 4, 5]),
    ];
    let mut node = MiniNode::new_with_validator_updates(0, validators.clone(), updates);
    let height_zero_ctx = node.context();
    let height_zero_actions = node.protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &height_zero_ctx,
    );
    let height_zero_hash = match height_zero_actions.get(1) {
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { block_hash, .. },
        }) => *block_hash,
        _ => panic!("expected prepare action"),
    };
    for sender in [1u64, 2, 3] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Prepare {
                sequence: 410 + sender,
                height: 0,
                round: 0,
                block_hash: height_zero_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_zero_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    for sender in [1u64, 2, 3] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Commit {
                sequence: 420 + sender,
                height: 0,
                round: 0,
                block_hash: height_zero_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_zero_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    let wal_after_height_zero = node.protocol.consensus_wal();
    node.protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        wal_after_height_zero,
    );
    let height_one_ctx = node.context();
    let block_one = Block::new(1, 2, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let block_one_hash = block_one.compute_hash();
    let pre_prepare_one_actions = node.protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 430,
            height: 1,
            round: 0,
            block: block_one,
        }),
        &height_one_ctx,
    );
    node.apply_local_mutations(&pre_prepare_one_actions);
    for sender in [2u64, 3, 4] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Prepare {
                sequence: 440 + sender,
                height: 1,
                round: 0,
                block_hash: block_one_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_one_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    for sender in [2u64, 3, 4] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Commit {
                sequence: 450 + sender,
                height: 1,
                round: 0,
                block_hash: block_one_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_one_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    let wal_after_height_one = node.protocol.consensus_wal();
    node.protocol = IbftProtocol::from_wal(
        validators,
        Box::new(MockSignatureScheme::new(0)),
        wal_after_height_one,
    );
    let height_two_ctx = node.context();
    let stale_pre_prepare = node.protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 460,
            height: 2,
            round: 0,
            block: Block::new(2, 3, vec![], [0u8; 32], BLOCK_GAS_LIMIT),
        }),
        &height_two_ctx,
    );
    let block_two = Block::new(2, 4, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let block_two_hash = block_two.compute_hash();

    // Act
    let pre_prepare_two_actions = node.protocol.handle_message(
        &MessageSource::Peer(4),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 461,
            height: 2,
            round: 0,
            block: block_two.clone(),
        }),
        &height_two_ctx,
    );
    node.apply_local_mutations(&pre_prepare_two_actions);
    for sender in [2u64, 3, 5] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Prepare {
                sequence: 470 + sender,
                height: 2,
                round: 0,
                block_hash: block_two_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_two_ctx,
        );
        node.apply_local_mutations(&actions);
    }
    for sender in [2u64, 3, 5] {
        let actions = node.protocol.handle_message(
            &MessageSource::Peer(sender),
            &Message::Peer(IbftMessage::Commit {
                sequence: 480 + sender,
                height: 2,
                round: 0,
                block_hash: block_two_hash,
                sender_signature: SignatureBytes::zeroed(),
            }),
            &height_two_ctx,
        );
        node.apply_local_mutations(&actions);
    }

    // Assert
    assert_eq!(stale_pre_prepare.len(), 0);
    assert_eq!(node.height, 3);
    assert_eq!(node.stored_block, Some(block_two));
}

#[test]
fn wal_serialize_round_trip_restored_node_completes_next_consensus_height() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut nodes = vec![
        MiniNode::new(0, validators.clone()),
        MiniNode::new(1, validators.clone()),
        MiniNode::new(2, validators.clone()),
        MiniNode::new(3, validators.clone()),
    ];
    let mut queue = VecDeque::new();
    let ctx0 = nodes[0].context();
    let actions0 = nodes[0].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx0,
    );
    queue.extend(queue_broadcasts(0, &actions0, &validators));
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }
    assert!(nodes.iter().all(|n| n.height == 1));
    let raw_wal = nodes[0].protocol.consensus_wal();
    let bytes = raw_wal.to_bytes();
    let deserialized = ConsensusWal::from_bytes(&bytes).expect("from_bytes failed");
    nodes[0].protocol = IbftProtocol::from_wal(
        validators.clone(),
        Box::new(MockSignatureScheme::new(0)),
        deserialized,
    );

    // Act
    let proposer_index = find_index(&nodes, 1);
    let ctx1 = nodes[proposer_index].context();
    let actions1 = nodes[proposer_index].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx1,
    );
    queue.extend(queue_broadcasts(1, &actions1, &validators));
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }

    // Assert
    for node in &nodes {
        assert_eq!(node.height, 2);
    }
}

#[test]
fn all_nodes_restart_after_height_zero_then_height_one_advances_all_to_height_two() {
    // Arrange
    let validators = vec![0, 1, 2, 3];
    let mut nodes = vec![
        MiniNode::new(0, validators.clone()),
        MiniNode::new(1, validators.clone()),
        MiniNode::new(2, validators.clone()),
        MiniNode::new(3, validators.clone()),
    ];
    let mut queue = VecDeque::new();
    let ctx0 = nodes[0].context();
    let actions0 = nodes[0].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx0,
    );
    queue.extend(queue_broadcasts(0, &actions0, &validators));
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }
    let block_zero = Block::new(0, 0, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let hash_zero = block_zero.compute_hash();
    for from in 0..4u64 {
        for to in 0..4u64 {
            if from != to {
                queue.push_back((
                    from,
                    to,
                    IbftMessage::Commit {
                        sequence: 900 + from,
                        height: 0,
                        round: 0,
                        block_hash: hash_zero,
                        sender_signature: SignatureBytes::zeroed(),
                    },
                ));
            }
        }
    }
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
    }
    assert!(nodes.iter().all(|n| n.height == 1));
    for node in &mut nodes {
        let wal = node.protocol.consensus_wal();
        node.protocol = IbftProtocol::from_wal(
            validators.clone(),
            Box::new(MockSignatureScheme::new(0)),
            wal,
        );
    }

    // Act
    let proposer_index = find_index(&nodes, 1);
    let ctx1 = nodes[proposer_index].context();
    let actions1 = nodes[proposer_index].protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx1,
    );
    queue.extend(queue_broadcasts(1, &actions1, &validators));
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
        queue.extend(queue_broadcasts(to, &actions, &validators));
    }
    let block_one = Block::new(1, 1, vec![], [0u8; 32], BLOCK_GAS_LIMIT);
    let hash_one = block_one.compute_hash();
    for from in 0..4u64 {
        for to in 0..4u64 {
            if from != to {
                queue.push_back((
                    from,
                    to,
                    IbftMessage::Commit {
                        sequence: 910 + from,
                        height: 1,
                        round: 0,
                        block_hash: hash_one,
                        sender_signature: SignatureBytes::zeroed(),
                    },
                ));
            }
        }
    }
    while let Some((from, to, message)) = queue.pop_front() {
        let index = find_index(&nodes, to);
        let ctx = nodes[index].context();
        let actions = nodes[index].protocol.handle_message(
            &MessageSource::Peer(from),
            &Message::Peer(message),
            &ctx,
        );
        nodes[index].apply_local_mutations(&actions);
    }

    // Assert
    for node in &nodes {
        assert_eq!(node.height, 2);
    }
}
