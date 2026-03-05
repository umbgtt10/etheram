// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_variants::implementations::ibft::ed25519_signature_scheme::Ed25519SignatureScheme;
use etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_variants::implementations::ibft::signature_scheme::SignatureScheme;

#[test]
fn sign_produces_non_zeroed_signature() {
    // Arrange
    let scheme = Ed25519SignatureScheme::new(0);

    // Act
    let sig = scheme.sign(b"test payload");

    // Assert
    assert_ne!(sig, SignatureBytes::zeroed());
}

#[test]
fn sign_and_verify_same_peer_returns_true() {
    // Arrange
    let scheme = Ed25519SignatureScheme::new(0);
    let data = b"consensus payload";

    // Act
    let sig = scheme.sign(data);
    let valid = scheme.verify_for_peer(data, &sig, 0);

    // Assert
    assert!(valid);
}

#[test]
fn sign_and_verify_cross_peer_returns_true() {
    // Arrange
    let signing_scheme = Ed25519SignatureScheme::new(0);
    let verifying_scheme = Ed25519SignatureScheme::new(1);
    let data = b"cross peer payload";

    // Act
    let sig = signing_scheme.sign(data);
    let valid = verifying_scheme.verify_for_peer(data, &sig, 0);

    // Assert
    assert!(valid);
}

#[test]
fn verify_for_peer_wrong_peer_id_returns_false() {
    // Arrange
    let scheme = Ed25519SignatureScheme::new(0);
    let data = b"payload";

    // Act
    let sig = scheme.sign(data);
    let valid = scheme.verify_for_peer(data, &sig, 1);

    // Assert
    assert!(!valid);
}

#[test]
fn verify_for_peer_tampered_signature_returns_false() {
    // Arrange
    let scheme = Ed25519SignatureScheme::new(0);
    let data = b"payload";
    let mut sig = scheme.sign(data);
    let tampered_bytes = {
        let mut b = *sig.as_bytes();
        b[0] ^= 0xff;
        b
    };
    sig = SignatureBytes::from_slice(&tampered_bytes);

    // Act
    let valid = scheme.verify_for_peer(data, &sig, 0);

    // Assert
    assert!(!valid);
}

#[test]
fn verify_for_peer_zeroed_signature_returns_false() {
    // Arrange
    let scheme = Ed25519SignatureScheme::new(0);

    // Act
    let valid = scheme.verify_for_peer(b"payload", &SignatureBytes::zeroed(), 0);

    // Assert
    assert!(!valid);
}

#[test]
fn sign_different_peer_ids_produce_different_signatures() {
    // Arrange
    let scheme_0 = Ed25519SignatureScheme::new(0);
    let scheme_1 = Ed25519SignatureScheme::new(1);
    let data = b"same payload";

    // Act
    let sig_0 = scheme_0.sign(data);
    let sig_1 = scheme_1.sign(data);

    // Assert
    assert_ne!(sig_0, sig_1);
}

#[test]
fn verify_for_peer_wrong_data_returns_false() {
    // Arrange
    let scheme = Ed25519SignatureScheme::new(2);
    let data = b"original";

    // Act
    let sig = scheme.sign(data);
    let valid = scheme.verify_for_peer(b"tampered", &sig, 2);

    // Assert
    assert!(!valid);
}
