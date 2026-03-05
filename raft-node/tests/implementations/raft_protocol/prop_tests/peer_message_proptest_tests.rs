// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_context::make_ctx_with_term;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use proptest::prelude::*;
use raft_node::brain::protocol::action::RaftAction;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::brain::protocol::unified_message::Message;
use raft_node::common_types::node_role::NodeRole;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;

fn arb_peer_id() -> BoxedStrategy<u64> {
    (1u64..=9u64).boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn request_vote_from_lower_term_always_rejects(
        peer_id in arb_peer_id(),
        ctx_term in 5u64..=15u64,
        msg_term in 1u64..5u64,
        sender in arb_peer_id(),
    ) {
        // Arrange
        let mut protocol = RaftProtocol::<Vec<u8>>::new();
        let ctx = make_ctx_with_term(peer_id, vec![10, 11], NodeRole::Follower, ctx_term);
        let message = Message::Peer(RaftMessage::RequestVote {
            term: msg_term,
            candidate_id: sender,
            last_log_index: 0,
            last_log_term: 0,
        });

        // Act
        let actions = protocol.handle_message(&MessageSource::Peer(sender), &message, &ctx);

        // Assert
        let rejects = actions.iter().any(|a| matches!(
            a,
            RaftAction::SendMessage {
                message: RaftMessage::RequestVoteResponse { vote_granted: false, .. },
                ..
            }
        ));
        prop_assert!(rejects);
    }

    #[test]
    fn append_entries_from_lower_term_always_returns_failure(
        peer_id in arb_peer_id(),
        ctx_term in 5u64..=15u64,
        msg_term in 1u64..5u64,
        sender in arb_peer_id(),
    ) {
        // Arrange
        let mut protocol = RaftProtocol::<Vec<u8>>::new();
        let ctx = make_ctx_with_term(peer_id, vec![10, 11], NodeRole::Follower, ctx_term);
        let message = Message::Peer(RaftMessage::AppendEntries {
            term: msg_term,
            leader_id: sender,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: alloc::vec![],
            leader_commit: 0,
        });

        // Act
        let actions = protocol.handle_message(&MessageSource::Peer(sender), &message, &ctx);

        // Assert
        let rejects = actions.iter().any(|a| matches!(
            a,
            RaftAction::SendMessage {
                message: RaftMessage::AppendEntriesResponse { success: false, .. },
                ..
            }
        ));
        prop_assert!(rejects);
    }

    #[test]
    fn higher_term_request_vote_always_produces_set_term(
        peer_id in arb_peer_id(),
        ctx_term in 1u64..=10u64,
        term_delta in 1u64..=20u64,
        sender in arb_peer_id(),
    ) {
        // Arrange
        let mut protocol = RaftProtocol::<Vec<u8>>::new();
        let ctx = make_ctx_with_term(peer_id, vec![10, 11], NodeRole::Follower, ctx_term);
        let msg_term = ctx_term + term_delta;
        let message = Message::Peer(RaftMessage::RequestVote {
            term: msg_term,
            candidate_id: sender,
            last_log_index: 0,
            last_log_term: 0,
        });

        // Act
        let actions = protocol.handle_message(&MessageSource::Peer(sender), &message, &ctx);

        // Assert
        let has_set_term = actions.iter().any(|a| matches!(
            a,
            RaftAction::SetTerm(t) if *t == msg_term
        ));
        prop_assert!(has_set_term);
    }
}
