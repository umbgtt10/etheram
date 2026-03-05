// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::signature_scheme::BoxedSignatureScheme;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::implementations::ibft::signature_scheme::SignatureScheme;

struct RejectScheme;

impl SignatureScheme for RejectScheme {
    type Signature = SignatureBytes;

    fn sign(&self, _data: &[u8]) -> SignatureBytes {
        SignatureBytes::from_slice(&[0xBEu8; 96])
    }

    fn verify_for_peer(&self, _data: &[u8], _sig: &SignatureBytes, _peer: PeerId) -> bool {
        false
    }
}

#[test]
fn sign_returns_expected_signature_bytes() {
    // Arrange
    let scheme = RejectScheme;

    // Act
    let sig = scheme.sign(b"payload");

    // Assert
    assert_eq!(sig, SignatureBytes::from_slice(&[0xBEu8; 96]));
}

#[test]
fn verify_for_peer_returns_false_for_reject_scheme() {
    // Arrange
    let scheme = RejectScheme;
    let sig = SignatureBytes::zeroed();

    // Act
    let result = scheme.verify_for_peer(b"payload", &sig, 0);

    // Assert
    assert!(!result);
}

#[test]
fn boxed_scheme_sign_dispatches_through_trait_object() {
    // Arrange
    let scheme: BoxedSignatureScheme = Box::new(RejectScheme);

    // Act
    let sig = scheme.sign(b"data");

    // Assert
    assert_eq!(sig, SignatureBytes::from_slice(&[0xBEu8; 96]));
}

#[test]
fn boxed_scheme_verify_dispatches_through_trait_object() {
    // Arrange
    let scheme: BoxedSignatureScheme = Box::new(RejectScheme);
    let sig = SignatureBytes::zeroed();

    // Act
    let result = scheme.verify_for_peer(b"data", &sig, 42);

    // Assert
    assert!(!result);
}
