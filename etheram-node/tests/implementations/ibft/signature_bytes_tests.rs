// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;

#[test]
fn from_slice_shorter_than_96_bytes_copies_and_zero_pads() {
    // Arrange
    let src = [0xABu8; 64];

    // Act
    let sig = SignatureBytes::from_slice(&src);

    // Assert
    let bytes = sig.as_bytes();
    assert_eq!(&bytes[..64], &src);
    assert_eq!(&bytes[64..], &[0u8; 32]);
}

#[test]
fn from_slice_exactly_96_bytes_copies_all() {
    // Arrange
    let src = [0x55u8; 96];

    // Act
    let sig = SignatureBytes::from_slice(&src);

    // Assert
    assert_eq!(sig.as_bytes(), &src);
}

#[test]
fn from_slice_longer_than_96_bytes_truncates_to_96() {
    // Arrange
    let mut src = [0xFFu8; 128];
    src[96] = 0x01;

    // Act
    let sig = SignatureBytes::from_slice(&src);

    // Assert
    assert_eq!(sig.as_bytes(), &[0xFFu8; 96]);
}

#[test]
fn from_slice_empty_returns_zeroed() {
    // Arrange & Act
    let sig = SignatureBytes::from_slice(&[]);

    // Assert
    assert_eq!(sig, SignatureBytes::zeroed());
}

#[test]
fn as_bytes_round_trip_returns_original_data() {
    // Arrange
    let mut src = [0u8; 96];
    src[0] = 0x01;
    src[47] = 0x7F;
    src[95] = 0xFF;

    // Act
    let sig = SignatureBytes::from_slice(&src);

    // Assert
    assert_eq!(sig.as_bytes(), &src);
}
