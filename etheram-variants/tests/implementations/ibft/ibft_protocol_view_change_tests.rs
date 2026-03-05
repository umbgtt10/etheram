// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_protocol;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::block::Block;
use etheram_node::context::context_dto::Context;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn handle_message_timeout_round_broadcasts_view_change() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::ViewChange {
                sequence: _,
                height: 0,
                round: 1,
                prepared_certificate: None,
            }
        })
    ));
}

#[test]
fn handle_message_view_change_quorum_on_new_proposer_broadcasts_new_view() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView {
                height: 0,
                round: 1,
                prepared_certificate: None,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_view_change_quorum_new_view_contains_deterministic_sender_order() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(3), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView {
                height: 0,
                round: 1,
                view_change_senders,
                ..
            }
        }) if *view_change_senders == vec![0, 1, 3]
    ));
}

#[test]
fn handle_message_new_view_round_one_allows_round_one_pre_prepare() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });
    protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);
    let block = Block::new(0, 1, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 1,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 0,
                round: 1,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_timeout_then_round_one_proposer_timer_broadcasts_round_one_messages() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

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
            message: IbftMessage::PrePrepare {
                height: 0,
                round: 1,
                ..
            }
        })
    ));
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 0,
                round: 1,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_timeout_then_round_zero_pre_prepare_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 0,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_wrong_proposer_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_below_quorum_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_stale_round_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 0,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_stale_round_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 0,
        prepared_certificate: None,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_quorum_not_on_proposer_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_wrong_height_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 1,
        round: 1,
        prepared_certificate: None,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_from_non_validator_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(99), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_wrong_height_returns_empty() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(2, 0);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 1,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_timeout_round_twice_emits_second_view_change_round() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(3, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::ViewChange {
                sequence: _,
                height: 0,
                round: 2,
                prepared_certificate: None,
            }
        })
    ));
}

#[test]
fn handle_message_new_view_duplicate_view_change_senders_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 0, 1],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_non_validator_view_change_senders_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 99],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_invalid_prepared_certificate_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [1u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (0, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let invalid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid_view_change, &ctx);
    let valid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &valid_view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_invalid_prepared_certificate_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [2u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (0, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
        view_change_senders: vec![0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_quorum_reached_multiple_times_broadcasts_new_view_once() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);
    let first_new_view_actions =
        protocol.handle_message(&MessageSource::Peer(2), &view_change, &ctx);

    // Act
    let second_new_view_actions =
        protocol.handle_message(&MessageSource::Peer(3), &view_change, &ctx);

    // Assert
    assert_eq!(first_new_view_actions.len(), 1);
    assert_eq!(second_new_view_actions.len(), 0);
}

#[test]
fn handle_message_new_view_duplicate_view_change_senders_with_unique_quorum_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_prepared_certificate_non_validator_signer_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [4u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (99, SignatureBytes::zeroed()),
        ],
    };
    let invalid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid_view_change, &ctx);
    let valid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &valid_view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_prepared_certificate_wrong_height_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let invalid_prepared_certificate = PreparedCertificate {
        height: 1,
        round: 1,
        block_hash: [5u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
        view_change_senders: vec![0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_prepared_certificate_higher_round_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 2,
        block_hash: [6u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
        view_change_senders: vec![0, 1, 2],
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_prepared_certificate_duplicate_signers_with_unique_quorum_returns_empty(
) {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let invalid_prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [7u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let invalid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(invalid_prepared_certificate),
    });
    protocol.handle_message(&MessageSource::Peer(0), &invalid_view_change, &ctx);
    let valid_view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &valid_view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_new_view_prepared_certificate_signers_subset_allows_round_transition() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [9u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(prepared_certificate),
        view_change_senders: vec![0, 1, 2],
    });
    protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);
    let block = Block::new(0, 1, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 1,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 0,
                round: 1,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_view_change_same_round_conflicting_certificate_first_cert_wins_in_new_view() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let prepared_certificate_a = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [11u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let prepared_certificate_b = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [12u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let view_change_a = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(prepared_certificate_a),
    });
    let view_change_b = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(prepared_certificate_b),
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change_a, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &view_change_b, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView {
                prepared_certificate: Some(cert),
                ..
            }
        }) if cert.block_hash == [11u8; 32]
    ));
}

#[test]
fn handle_message_view_change_duplicate_sender_before_quorum_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_view_change_duplicate_sender_then_unique_sender_reaches_quorum_with_unique_sorted_senders(
) {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView {
                height: 0,
                round: 1,
                view_change_senders,
                ..
            }
        }) if *view_change_senders == vec![0, 1, 2]
    ));
}

#[test]
fn handle_message_view_change_duplicate_sender_after_quorum_broadcasted_returns_empty() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(1, 0, [0u8; 32]);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let view_change = Message::Peer(IbftMessage::ViewChange {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
    });
    protocol.handle_message(&MessageSource::Peer(0), &view_change, &ctx);
    protocol.handle_message(&MessageSource::Peer(2), &view_change, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(2), &view_change, &ctx);

    // Assert
    assert_eq!(actions.len(), 0);
}

#[test]
fn handle_message_propose_block_after_timeout_with_prepared_certificate_re_proposes_locked_block() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    let locked_block = Block::new(0, 0, vec![], [0u8; 32]);
    let locked_block_hash = locked_block.compute_hash();
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 10,
            height: 0,
            round: 0,
            block: locked_block.clone(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 20,
            height: 0,
            round: 0,
            block_hash: locked_block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(2),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 30,
            height: 0,
            round: 0,
            block_hash: locked_block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Peer(3),
        &Message::Peer(IbftMessage::Prepare {
            sequence: 40,
            height: 0,
            round: 0,
            block_hash: locked_block_hash,
            sender_signature: SignatureBytes::zeroed(),
        }),
        &ctx,
    );
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::PrePrepare {
                height: 0,
                round: 1,
                block: re_proposed,
                ..
            }
        }) if re_proposed.compute_hash() == locked_block_hash
    ));
}

#[test]
fn handle_message_view_change_higher_round_certificate_takes_precedence_over_lower_round() {
    // Arrange
    let mut protocol = setup_protocol();
    let ctx = setup_context(1, 0);
    protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::TimeoutRound),
        &ctx,
    );
    let vc_low = Message::Peer(IbftMessage::ViewChange {
        sequence: 100,
        height: 0,
        round: 1,
        prepared_certificate: Some(PreparedCertificate {
            height: 0,
            round: 0,
            block_hash: [1u8; 32],
            signed_prepares: vec![
                (0, SignatureBytes::zeroed()),
                (1, SignatureBytes::zeroed()),
                (2, SignatureBytes::zeroed()),
            ],
        }),
    });
    let vc_high = Message::Peer(IbftMessage::ViewChange {
        sequence: 101,
        height: 0,
        round: 1,
        prepared_certificate: Some(PreparedCertificate {
            height: 0,
            round: 1,
            block_hash: [2u8; 32],
            signed_prepares: vec![
                (0, SignatureBytes::zeroed()),
                (2, SignatureBytes::zeroed()),
                (3, SignatureBytes::zeroed()),
            ],
        }),
    });
    protocol.handle_message(&MessageSource::Peer(0), &vc_low, &ctx);

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(3), &vc_high, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::NewView {
                height: 0,
                round: 1,
                prepared_certificate: Some(cert),
                ..
            }
        }) if cert.block_hash == [2u8; 32]
    ));
}

#[test]
fn handle_message_new_view_cert_signers_not_subset_of_view_change_senders_accepts() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let prepared_certificate = PreparedCertificate {
        height: 0,
        round: 1,
        block_hash: [8u8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (1, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
        ],
    };
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: Some(prepared_certificate),
        view_change_senders: vec![0, 1, 3],
    });
    protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);
    let block = Block::new(0, 1, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 1,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 0,
                round: 1,
                ..
            }
        })
    ));
}

#[test]
fn handle_message_new_view_without_local_view_change_votes_accepts() {
    // Arrange
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(0)));
    let ctx = Context::new(2, 0, [0u8; 32]);
    let new_view = Message::Peer(IbftMessage::NewView {
        sequence: 0,
        height: 0,
        round: 1,
        prepared_certificate: None,
        view_change_senders: vec![0, 1, 3],
    });
    protocol.handle_message(&MessageSource::Peer(1), &new_view, &ctx);
    let block = Block::new(0, 1, vec![], [0u8; 32]);
    let pre_prepare = Message::Peer(IbftMessage::PrePrepare {
        sequence: 0,
        height: 0,
        round: 1,
        block,
    });

    // Act
    let actions = protocol.handle_message(&MessageSource::Peer(1), &pre_prepare, &ctx);

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                height: 0,
                round: 1,
                ..
            }
        })
    ));
}
