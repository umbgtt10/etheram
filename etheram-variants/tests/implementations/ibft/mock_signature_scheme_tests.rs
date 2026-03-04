// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_etheram_variants::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_etheram_variants::implementations::ibft::signature_scheme::SignatureScheme;

#[test]
fn sign_any_data_returns_zeroed_signature_bytes() {
    // Arrange
    let scheme = MockSignatureScheme::new(0);

    // Act & Assert
    assert_eq!(scheme.sign(b"any payload"), SignatureBytes::zeroed());
}

#[test]
fn verify_for_peer_any_input_returns_true() {
    // Arrange
    let scheme = MockSignatureScheme::new(0);

    // Act
    let result = scheme.verify_for_peer(b"any payload", &SignatureBytes::zeroed(), 0);

    // Assert
    assert!(result);
}
