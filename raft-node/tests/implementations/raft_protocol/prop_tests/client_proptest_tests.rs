// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::make_ctx;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::node_common::action_collection::ActionCollection;
use proptest::prelude::*;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::brain::protocol::unified_message::Message;
use raft_node::common_types::node_role::NodeRole;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;

fn arb_peer_id() -> BoxedStrategy<u64> {
    (1u64..=9u64).boxed()
}

fn arb_payload() -> BoxedStrategy<Vec<u8>> {
    any::<u8>().prop_map(|b| vec![b]).boxed()
}

fn arb_key() -> BoxedStrategy<String> {
    "[a-z]{1,8}".boxed()
}

fn send_command(
    peer_id: u64,
    role: NodeRole,
    payload: Vec<u8>,
) -> ActionCollection<RaftAction<Vec<u8>>> {
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(peer_id, vec![10, 11], role);
    protocol.handle_message(
        &MessageSource::Client(42),
        &Message::Client(RaftClientRequest::Command(payload)),
        &ctx,
    )
}

fn send_query(peer_id: u64, role: NodeRole, key: String) -> ActionCollection<RaftAction<Vec<u8>>> {
    let mut protocol = RaftProtocol::<Vec<u8>>::new();
    let ctx = make_ctx(peer_id, vec![10, 11], role);
    protocol.handle_message(
        &MessageSource::Client(42),
        &Message::Client(RaftClientRequest::Query(key)),
        &ctx,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn command_as_follower_always_returns_not_leader(
        peer_id in arb_peer_id(),
        payload in arb_payload(),
    ) {
        // Arrange & Act
        let actions = send_command(peer_id, NodeRole::Follower, payload);

        // Assert
        let is_not_leader = matches!(
            actions.get(0),
            Some(RaftAction::SendClientResponse {
                response: RaftClientResponse::NotLeader(_),
                ..
            })
        );
        prop_assert!(is_not_leader);
        prop_assert_eq!(actions.len(), 1);
    }

    #[test]
    fn query_as_follower_always_returns_not_leader(
        peer_id in arb_peer_id(),
        key in arb_key(),
    ) {
        // Arrange & Act
        let actions = send_query(peer_id, NodeRole::Follower, key);

        // Assert
        let is_not_leader = matches!(
            actions.get(0),
            Some(RaftAction::SendClientResponse {
                response: RaftClientResponse::NotLeader(_),
                ..
            })
        );
        prop_assert!(is_not_leader);
        prop_assert_eq!(actions.len(), 1);
    }

    #[test]
    fn command_as_leader_always_appends_entries(
        peer_id in arb_peer_id(),
        payload in arb_payload(),
    ) {
        // Arrange & Act
        let actions = send_command(peer_id, NodeRole::Leader, payload);

        // Assert
        let has_append = actions.iter().any(|a| matches!(a, RaftAction::AppendEntries(_)));
        prop_assert!(has_append);
    }

    #[test]
    fn query_as_leader_always_produces_exactly_one_query_state_machine(
        peer_id in arb_peer_id(),
        key in arb_key(),
    ) {
        // Arrange & Act
        let actions = send_query(peer_id, NodeRole::Leader, key.clone());

        // Assert
        let query_count = actions.iter().filter(|a| matches!(
            a,
            RaftAction::QueryStateMachine { key: k, .. } if k == &key
        )).count();
        prop_assert_eq!(query_count, 1);
    }

    #[test]
    fn command_as_candidate_always_returns_not_leader(
        peer_id in arb_peer_id(),
        payload in arb_payload(),
    ) {
        // Arrange & Act
        let actions = send_command(peer_id, NodeRole::Candidate, payload);

        // Assert
        let is_not_leader = matches!(
            actions.get(0),
            Some(RaftAction::SendClientResponse {
                response: RaftClientResponse::NotLeader(_),
                ..
            })
        );
        prop_assert!(is_not_leader);
        prop_assert_eq!(actions.len(), 1);
    }
}
