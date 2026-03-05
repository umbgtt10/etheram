// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::transport_outgoing::TransportOutgoing;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::implementations::no_op_raft_transport::NoOpRaftTransport;

#[test]
fn poll_returns_none() {
    // Arrange
    let transport = NoOpRaftTransport::<Vec<u8>>::new();

    // Act
    let message = transport.poll();

    // Assert
    assert!(message.is_none());
}

#[test]
fn send_any_message_does_not_panic() {
    // Arrange
    let transport = NoOpRaftTransport::<Vec<u8>>::new();

    // Act
    transport.send(
        2,
        RaftMessage::RequestVote {
            term: 1,
            candidate_id: 1,
            last_log_index: 0,
            last_log_term: 0,
        },
    );
    let message = transport.poll();

    // Assert
    assert!(message.is_none());
}
