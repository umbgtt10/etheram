// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use barechain_etheram_variants::implementations::tiny_evm_engine::opcode_name;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_ADD;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_PUSH1;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_RETURN;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_SLOAD;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_SSTORE;
use barechain_etheram_variants::implementations::tiny_evm_engine::OPCODE_STOP;

#[test]
fn opcode_name_stop_returns_expected_name() {
    // Act & Assert
    assert_eq!(opcode_name(OPCODE_STOP), Some("STOP"));
}

#[test]
fn opcode_name_add_returns_expected_name() {
    // Act & Assert
    assert_eq!(opcode_name(OPCODE_ADD), Some("ADD"));
}

#[test]
fn opcode_name_sload_returns_expected_name() {
    // Act & Assert
    assert_eq!(opcode_name(OPCODE_SLOAD), Some("SLOAD"));
}

#[test]
fn opcode_name_sstore_returns_expected_name() {
    // Act & Assert
    assert_eq!(opcode_name(OPCODE_SSTORE), Some("SSTORE"));
}

#[test]
fn opcode_name_push1_returns_expected_name() {
    // Act & Assert
    assert_eq!(opcode_name(OPCODE_PUSH1), Some("PUSH1"));
}

#[test]
fn opcode_name_return_returns_expected_name() {
    // Act & Assert
    assert_eq!(opcode_name(OPCODE_RETURN), Some("RETURN"));
}

#[test]
fn opcode_name_unknown_opcode_returns_none() {
    // Act & Assert
    assert_eq!(opcode_name(0xff), None);
}

#[test]
fn opcode_constants_match_ethereum_specification() {
    // Act & Assert
    assert_eq!(OPCODE_STOP, 0x00);
    assert_eq!(OPCODE_ADD, 0x01);
    assert_eq!(OPCODE_SLOAD, 0x54);
    assert_eq!(OPCODE_SSTORE, 0x55);
    assert_eq!(OPCODE_PUSH1, 0x60);
    assert_eq!(OPCODE_RETURN, 0xf3);
}
