// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub const OPCODE_STOP: u8 = 0x00;
pub const OPCODE_ADD: u8 = 0x01;
pub const OPCODE_SLOAD: u8 = 0x54;
pub const OPCODE_SSTORE: u8 = 0x55;
pub const OPCODE_PUSH1: u8 = 0x60;
pub const OPCODE_RETURN: u8 = 0xf3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TinyEvmOpcode {
    Stop,
    Add,
    Sload,
    Sstore,
    Push1,
    Return,
}

pub fn opcode_name(byte: u8) -> Option<&'static str> {
    match byte {
        OPCODE_STOP => Some("STOP"),
        OPCODE_ADD => Some("ADD"),
        OPCODE_SLOAD => Some("SLOAD"),
        OPCODE_SSTORE => Some("SSTORE"),
        OPCODE_PUSH1 => Some("PUSH1"),
        OPCODE_RETURN => Some("RETURN"),
        _ => None,
    }
}

pub(crate) fn decode_opcode(byte: u8) -> Option<TinyEvmOpcode> {
    match byte {
        OPCODE_STOP => Some(TinyEvmOpcode::Stop),
        OPCODE_ADD => Some(TinyEvmOpcode::Add),
        OPCODE_SLOAD => Some(TinyEvmOpcode::Sload),
        OPCODE_SSTORE => Some(TinyEvmOpcode::Sstore),
        OPCODE_PUSH1 => Some(TinyEvmOpcode::Push1),
        OPCODE_RETURN => Some(TinyEvmOpcode::Return),
        _ => None,
    }
}
