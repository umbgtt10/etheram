// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::common::ibft_protocol_test_helpers::setup_context;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_core::types::PeerId;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::block::Block;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_variants::implementations::ibft::ed25519_signature_scheme::Ed25519SignatureScheme;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_variants::implementations::ibft::signature_scheme::SignatureScheme;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

struct CountingSignatureScheme {
    sign_calls: Arc<AtomicUsize>,
}

impl SignatureScheme for CountingSignatureScheme {
    type Signature = SignatureBytes;

    fn sign(&self, _data: &[u8]) -> SignatureBytes {
        self.sign_calls.fetch_add(1, Ordering::SeqCst);
        SignatureBytes::zeroed()
    }

    fn verify_for_peer(&self, _data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
        true
    }
}

#[test]
fn handle_message_timer_propose_block_creates_inner_prepare_signature() {
    // Arrange
    let sign_calls = Arc::new(AtomicUsize::new(0));
    let scheme = CountingSignatureScheme {
        sign_calls: Arc::clone(&sign_calls),
    };
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(scheme));
    let ctx = setup_context(0, 0);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Timer,
        &Message::Timer(TimerEvent::ProposeBlock),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 2);
    assert_eq!(sign_calls.load(Ordering::SeqCst), 1);
    assert!(matches!(
        actions.get(1),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare {
                sender_signature, ..
            }
        }) if *sender_signature == SignatureBytes::zeroed()
    ));
}

#[test]
fn handle_message_peer_pre_prepare_accepted_without_outer_auth() {
    // Arrange
    let mut protocol =
        IbftProtocol::new(vec![0, 1, 2, 3], Box::new(Ed25519SignatureScheme::new(1)));
    let ctx = setup_context(1, 0);
    let block = Block::new(0, 0, vec![], [0u8; 32]);

    // Act
    let actions = protocol.handle_message(
        &MessageSource::Peer(0),
        &Message::Peer(IbftMessage::PrePrepare {
            sequence: 1,
            height: 0,
            round: 0,
            block,
        }),
        &ctx,
    );

    // Assert
    assert_eq!(actions.len(), 1);
    assert!(matches!(
        actions.get(0),
        Some(Action::BroadcastMessage {
            message: IbftMessage::Prepare { .. }
        })
    ));
}

#[test]
fn handle_message_new_view_with_cert_invalid_sigs_does_not_advance_round() {
    // Arrange
    struct CertSigRejectingScheme;
    impl SignatureScheme for CertSigRejectingScheme {
        type Signature = SignatureBytes;
        fn sign(&self, _data: &[u8]) -> SignatureBytes {
            SignatureBytes::zeroed()
        }
        fn verify_for_peer(&self, data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
            data.first() != Some(&1)
        }
    }
    let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(CertSigRejectingScheme));
    let ctx = setup_context(3, 0);
    let cert = PreparedCertificate {
        height: 0,
        round: 0,
        block_hash: [0xabu8; 32],
        signed_prepares: vec![
            (0, SignatureBytes::zeroed()),
            (2, SignatureBytes::zeroed()),
            (3, SignatureBytes::zeroed()),
        ],
    };

    // Act
    let _ = protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::NewView {
            sequence: 1,
            height: 0,
            round: 1,
            prepared_certificate: Some(cert),
            view_change_senders: vec![0, 2, 3],
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.round, 0);
    assert!(wal.prepared_certificate.is_none());
}

#[test]
fn handle_message_ed25519_inner_prepare_signature_verified_in_certificate() {
    // Arrange
    let block = Block::new(0, 0, vec![], [0u8; 32]);
    let block_hash = block.compute_hash();
    let prepare_payload = {
        let mut payload = Vec::with_capacity(1 + 8 + 8 + 32);
        payload.push(1);
        payload.extend_from_slice(&0u64.to_le_bytes());
        payload.extend_from_slice(&0u64.to_le_bytes());
        payload.extend_from_slice(&block_hash);
        payload
    };
    let cert = PreparedCertificate {
        height: 0,
        round: 0,
        block_hash,
        signed_prepares: vec![
            (0, Ed25519SignatureScheme::new(0).sign(&prepare_payload)),
            (2, Ed25519SignatureScheme::new(2).sign(&prepare_payload)),
            (3, Ed25519SignatureScheme::new(3).sign(&prepare_payload)),
        ],
    };
    let mut protocol =
        IbftProtocol::new(vec![0, 1, 2, 3], Box::new(Ed25519SignatureScheme::new(3)));
    let ctx = setup_context(3, 0);

    // Act
    let _ = protocol.handle_message(
        &MessageSource::Peer(1),
        &Message::Peer(IbftMessage::NewView {
            sequence: 1,
            height: 0,
            round: 1,
            prepared_certificate: Some(cert),
            view_change_senders: vec![0, 2, 3],
        }),
        &ctx,
    );
    let wal = protocol.consensus_wal();

    // Assert
    assert_eq!(wal.round, 1);
    assert!(wal.prepared_certificate.is_some());
}
