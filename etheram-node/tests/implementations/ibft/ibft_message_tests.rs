// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn pre_prepare_construction_fields_match() {
    // Arrange
    let block = Block::empty(3, 0, [0u8; 32]);

    // Act
    let msg = IbftMessage::PrePrepare {
        sequence: 1,
        height: 3,
        round: 2,
        block: block.clone(),
    };

    // Assert
    if let IbftMessage::PrePrepare {
        sequence,
        height,
        round,
        block: b,
    } = msg
    {
        assert_eq!(sequence, 1);
        assert_eq!(height, 3);
        assert_eq!(round, 2);
        assert_eq!(b, block);
    } else {
        panic!("expected PrePrepare");
    }
}

#[test]
fn prepare_construction_fields_match() {
    // Arrange
    let hash = [0xABu8; 32];
    let sig = SignatureBytes::from_slice(&[0x01u8; 96]);

    // Act
    let msg = IbftMessage::Prepare {
        sequence: 5,
        height: 10,
        round: 1,
        block_hash: hash,
        sender_signature: sig,
    };

    // Assert
    if let IbftMessage::Prepare {
        sequence,
        height,
        round,
        block_hash,
        sender_signature,
    } = msg
    {
        assert_eq!(sequence, 5);
        assert_eq!(height, 10);
        assert_eq!(round, 1);
        assert_eq!(block_hash, hash);
        assert_eq!(sender_signature, sig);
    } else {
        panic!("expected Prepare");
    }
}

#[test]
fn commit_construction_fields_match() {
    // Arrange
    let hash = [0xCDu8; 32];
    let sig = SignatureBytes::zeroed();

    // Act
    let msg = IbftMessage::Commit {
        sequence: 7,
        height: 2,
        round: 0,
        block_hash: hash,
        sender_signature: sig,
    };

    // Assert
    if let IbftMessage::Commit {
        sequence,
        height,
        round,
        block_hash,
        sender_signature,
    } = msg
    {
        assert_eq!(sequence, 7);
        assert_eq!(height, 2);
        assert_eq!(round, 0);
        assert_eq!(block_hash, hash);
        assert_eq!(sender_signature, SignatureBytes::zeroed());
    } else {
        panic!("expected Commit");
    }
}

#[test]
fn view_change_without_certificate_stores_none() {
    // Arrange & Act
    let msg = IbftMessage::ViewChange {
        sequence: 0,
        height: 1,
        round: 2,
        prepared_certificate: None,
    };

    // Assert
    if let IbftMessage::ViewChange {
        prepared_certificate,
        ..
    } = msg
    {
        assert!(prepared_certificate.is_none());
    } else {
        panic!("expected ViewChange");
    }
}

#[test]
fn view_change_with_certificate_stores_some() {
    // Arrange
    let cert = PreparedCertificate {
        height: 1,
        round: 0,
        block_hash: [0u8; 32],
        signed_prepares: vec![],
    };

    // Act
    let msg = IbftMessage::ViewChange {
        sequence: 0,
        height: 1,
        round: 1,
        prepared_certificate: Some(cert.clone()),
    };

    // Assert
    if let IbftMessage::ViewChange {
        prepared_certificate,
        ..
    } = msg
    {
        assert_eq!(prepared_certificate, Some(cert));
    } else {
        panic!("expected ViewChange");
    }
}

#[test]
fn new_view_senders_count_matches() {
    // Arrange
    let senders = vec![1u64, 2, 3];

    // Act
    let msg = IbftMessage::NewView {
        sequence: 0,
        height: 5,
        round: 1,
        prepared_certificate: None,
        view_change_senders: senders.clone(),
    };

    // Assert
    if let IbftMessage::NewView {
        view_change_senders,
        ..
    } = msg
    {
        assert_eq!(view_change_senders, senders);
    } else {
        panic!("expected NewView");
    }
}

#[test]
fn clone_produces_equal_value() {
    // Arrange
    let msg = IbftMessage::Prepare {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash: [0u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    };

    // Act
    let cloned = msg.clone();

    // Assert
    assert_eq!(msg, cloned);
}

#[test]
fn different_rounds_are_not_equal() {
    // Arrange
    let a = IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 0,
        block_hash: [0u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    };
    let b = IbftMessage::Commit {
        sequence: 0,
        height: 0,
        round: 1,
        block_hash: [0u8; 32],
        sender_signature: SignatureBytes::zeroed(),
    };

    // Act & Assert
    assert_ne!(a, b);
}
