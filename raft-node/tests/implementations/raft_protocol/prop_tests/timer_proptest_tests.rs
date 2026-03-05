// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::make_ctx;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use proptest::prelude::*;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::brain::protocol::unified_message::Message;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

fn arb_peer_id() -> BoxedStrategy<u64> {
    (1u64..=9u64).boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn heartbeat_as_non_leader_always_returns_empty(
        peer_id in arb_peer_id(),
    ) {
        // Arrange
        let mut protocol = RaftProtocol::<Vec<u8>>::new();
        let ctx = make_ctx(peer_id, vec![10, 11], NodeRole::Follower);

        // Act
        let actions = protocol.handle_message(
            &MessageSource::Timer,
            &Message::Timer(RaftTimerEvent::Heartbeat),
            &ctx,
        );

        // Assert
        prop_assert_eq!(actions.len(), 0);
    }

    #[test]
    fn election_timeout_as_leader_reschedules_only(
        peer_id in arb_peer_id(),
    ) {
        // Arrange
        let mut protocol = RaftProtocol::<Vec<u8>>::new();
        let ctx = make_ctx(peer_id, vec![10, 11], NodeRole::Leader);

        // Act
        let actions = protocol.handle_message(
            &MessageSource::Timer,
            &Message::Timer(RaftTimerEvent::ElectionTimeout),
            &ctx,
        );

        // Assert
        let reschedule_count = actions.iter().filter(|a| matches!(
            a,
            RaftAction::ScheduleTimeout { event: RaftTimerEvent::ElectionTimeout, .. }
        )).count();
        let broadcast_count = actions.iter().filter(|a| matches!(
            a,
            RaftAction::BroadcastMessage { .. }
        )).count();
        prop_assert_eq!(reschedule_count, 1);
        prop_assert_eq!(broadcast_count, 0);
    }

    #[test]
    fn heartbeat_as_leader_sends_to_all_peers(
        peer_id in arb_peer_id(),
    ) {
        // Arrange
        let mut protocol = RaftProtocol::<Vec<u8>>::new();
        let peers = vec![10u64, 11, 12];
        let ctx = make_ctx(peer_id, peers.clone(), NodeRole::Leader);

        // Act
        let actions = protocol.handle_message(
            &MessageSource::Timer,
            &Message::Timer(RaftTimerEvent::Heartbeat),
            &ctx,
        );

        // Assert
        let send_count = actions.iter().filter(|a| matches!(a, RaftAction::SendMessage { .. })).count();
        prop_assert_eq!(send_count, peers.len());
    }
}
