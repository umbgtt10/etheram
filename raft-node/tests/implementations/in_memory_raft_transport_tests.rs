// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::standard_shared_state::StdSharedState;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::implementations::in_memory_raft_transport::InMemoryRaftTransport;
use raft_node::implementations::in_memory_raft_transport::InMemoryRaftTransportState;

#[test]
fn poll_empty_inbox_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new());
    let transport = InMemoryRaftTransport::new(1, state);

    // Act
    let message = transport.poll();

    // Assert
    assert!(message.is_none());
}

#[test]
fn send_message_to_peer_then_poll_returns_sender_and_message() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new());
    let sender = InMemoryRaftTransport::new(1, state.clone());
    let receiver = InMemoryRaftTransport::new(2, state);

    // Act
    sender.send(
        2,
        RaftMessage::RequestVote {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        },
    );
    let polled = receiver.poll();

    // Assert
    assert!(matches!(
        polled,
        Some((1, RaftMessage::RequestVote { term: 1, .. }))
    ));
}

#[test]
fn send_targets_correct_node() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new());
    let sender = InMemoryRaftTransport::new(1, state.clone());
    let receiver_2 = InMemoryRaftTransport::new(2, state.clone());
    let receiver_3 = InMemoryRaftTransport::new(3, state);

    // Act
    sender.send(
        3,
        RaftMessage::RequestVote {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        },
    );

    // Assert
    assert!(receiver_2.poll().is_none());
    assert!(receiver_3.poll().is_some());
}

#[test]
fn push_message_then_poll_returns_message_with_sender() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new());
    let transport = InMemoryRaftTransport::new(1, state.clone());
    state.with_mut(|s| {
        s.push_message(
            1,
            2,
            RaftMessage::RequestVote {
                term: 1,
                candidate_id: 2,
                last_log_index: 0,
                last_log_term: 0,
            },
        );
    });

    // Act
    let result = transport.poll();

    // Assert
    assert!(matches!(
        result,
        Some((2, RaftMessage::RequestVote { term: 1, .. }))
    ));
}

#[test]
fn push_message_different_node_id_not_visible_to_other_node() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new());
    let node_1 = InMemoryRaftTransport::new(1, state.clone());
    let _node_2 = InMemoryRaftTransport::new(2, state.clone());
    state.with_mut(|s| {
        s.push_message(
            2,
            1,
            RaftMessage::RequestVote {
                term: 1,
                candidate_id: 1,
                last_log_index: 0,
                last_log_term: 0,
            },
        );
    });

    // Act
    let result = node_1.poll();

    // Assert
    assert!(result.is_none());
}
