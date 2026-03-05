// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::context::context_dto::Context;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_node::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use proptest::prelude::*;

fn arb_context() -> BoxedStrategy<Context> {
    (0u64..4u64, 0u64..10u64)
        .prop_map(|(peer_id, height)| Context::new(peer_id, height, [0u8; 32]))
        .boxed()
}

fn arb_non_proposer_context() -> BoxedStrategy<Context> {
    (0u64..10u64)
        .prop_map(|height| {
            let proposer = height % 4;
            let peer_id = (proposer + 1) % 4;
            Context::new(peer_id, height, [0u8; 32])
        })
        .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn propose_block_by_non_proposer_produces_no_output(
        ctx in arb_non_proposer_context(),
    ) {
        // Arrange
        let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(ctx.peer_id)));

        // Act
        let actions = protocol.handle_message(
            &MessageSource::Timer,
            &Message::Timer(TimerEvent::ProposeBlock),
            &ctx,
        );

        // Assert
        prop_assert_eq!(actions.len(), 0);
    }

    #[test]
    fn timeout_round_always_produces_exactly_one_view_change(
        ctx in arb_context(),
    ) {
        // Arrange
        let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(ctx.peer_id)));

        // Act
        let actions = protocol.handle_message(
            &MessageSource::Timer,
            &Message::Timer(TimerEvent::TimeoutRound),
            &ctx,
        );

        // Assert
        let view_change_count = actions.iter().filter(|a| matches!(
            a,
            Action::BroadcastMessage { message: IbftMessage::ViewChange { .. } }
        )).count();
        prop_assert_eq!(view_change_count, 1);
    }
}
