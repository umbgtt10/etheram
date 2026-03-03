// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::AlternateSignatureScheme;
use barechain_core::collection::Collection;
use barechain_core::consensus_protocol::ConsensusProtocol;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram::brain::protocol::action::Action;
use etheram::brain::protocol::message::Message;
use etheram::brain::protocol::message_source::MessageSource;
use etheram::incoming::timer::timer_event::TimerEvent;

#[test]
fn handle_message_proposer_timer_with_injected_signature_scheme_broadcasts_pre_prepare_and_prepare()
{
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(AlternateSignatureScheme));
    let ctx = setup_context(0, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare { .. }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}
