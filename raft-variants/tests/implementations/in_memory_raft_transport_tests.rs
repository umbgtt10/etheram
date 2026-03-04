// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::standard_shared_state::StdSharedState;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use raft_node::brain::protocol::message::RaftMessage;
use raft_variants::implementations::in_memory_raft_transport::InMemoryRaftTransport;
use raft_variants::implementations::in_memory_raft_transport::InMemoryRaftTransportState;

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
