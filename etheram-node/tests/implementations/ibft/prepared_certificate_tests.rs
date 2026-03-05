// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn fields_are_accessible_after_construction() {
    // Arrange & Act
    let hash = [0x1Fu8; 32];
    let cert = PreparedCertificate {
        height: 4,
        round: 1,
        block_hash: hash,
        signed_prepares: vec![(0u64, SignatureBytes::zeroed())],
    };

    // Assert
    assert_eq!(cert.height, 4);
    assert_eq!(cert.round, 1);
    assert_eq!(cert.block_hash, hash);
    assert_eq!(cert.signed_prepares.len(), 1);
}

#[test]
fn signed_prepares_stores_all_entries() {
    // Arrange
    let entries = vec![
        (0u64, SignatureBytes::zeroed()),
        (1u64, SignatureBytes::from_slice(&[0x01u8; 96])),
        (2u64, SignatureBytes::from_slice(&[0x02u8; 96])),
    ];

    // Act
    let cert = PreparedCertificate {
        height: 0,
        round: 0,
        block_hash: [0u8; 32],
        signed_prepares: entries.clone(),
    };

    // Assert
    assert_eq!(cert.signed_prepares.len(), 3);
    assert_eq!(cert.signed_prepares[1].0, 1u64);
}

#[test]
fn clone_produces_equal_certificate() {
    // Arrange
    let cert = PreparedCertificate {
        height: 2,
        round: 3,
        block_hash: [0xFFu8; 32],
        signed_prepares: vec![(5u64, SignatureBytes::zeroed())],
    };

    // Act
    let cloned = cert.clone();

    // Assert
    assert_eq!(cert, cloned);
}

#[test]
fn different_round_produces_unequal_certificates() {
    // Arrange
    let a = PreparedCertificate {
        height: 1,
        round: 0,
        block_hash: [0u8; 32],
        signed_prepares: vec![],
    };
    let b = PreparedCertificate {
        height: 1,
        round: 1,
        block_hash: [0u8; 32],
        signed_prepares: vec![],
    };

    // Act & Assert
    assert_ne!(a, b);
}

#[test]
fn different_block_hash_produces_unequal_certificates() {
    // Arrange
    let a = PreparedCertificate {
        height: 0,
        round: 0,
        block_hash: [0x00u8; 32],
        signed_prepares: vec![],
    };
    let b = PreparedCertificate {
        height: 0,
        round: 0,
        block_hash: [0xFFu8; 32],
        signed_prepares: vec![],
    };

    // Act & Assert
    assert_ne!(a, b);
}
