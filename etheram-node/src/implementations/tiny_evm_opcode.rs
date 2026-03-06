// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub const OPCODE_STOP: u8 = 0x00;
pub const OPCODE_ADD: u8 = 0x01;
pub const OPCODE_MUL: u8 = 0x02;
pub const OPCODE_SUB: u8 = 0x03;
pub const OPCODE_DIV: u8 = 0x04;
pub const OPCODE_LT: u8 = 0x10;
pub const OPCODE_GT: u8 = 0x11;
pub const OPCODE_EQ: u8 = 0x14;
pub const OPCODE_ISZERO: u8 = 0x15;
pub const OPCODE_AND: u8 = 0x16;
pub const OPCODE_OR: u8 = 0x17;
pub const OPCODE_SHA3: u8 = 0x20;
pub const OPCODE_CALLER: u8 = 0x33;
pub const OPCODE_CALLVALUE: u8 = 0x34;
pub const OPCODE_CALLDATALOAD: u8 = 0x35;
pub const OPCODE_CALLDATASIZE: u8 = 0x36;
pub const OPCODE_POP: u8 = 0x50;
pub const OPCODE_MLOAD: u8 = 0x51;
pub const OPCODE_MSTORE: u8 = 0x52;
pub const OPCODE_SLOAD: u8 = 0x54;
pub const OPCODE_SSTORE: u8 = 0x55;
pub const OPCODE_JUMP: u8 = 0x56;
pub const OPCODE_JUMPI: u8 = 0x57;
pub const OPCODE_JUMPDEST: u8 = 0x5b;
pub const OPCODE_PUSH1: u8 = 0x60;
pub const OPCODE_PUSH2: u8 = 0x61;
pub const OPCODE_PUSH32: u8 = 0x7f;
pub const OPCODE_DUP1: u8 = 0x80;
pub const OPCODE_DUP16: u8 = 0x8f;
pub const OPCODE_SWAP1: u8 = 0x90;
pub const OPCODE_SWAP16: u8 = 0x9f;
pub const OPCODE_RETURN: u8 = 0xf3;
pub const OPCODE_REVERT: u8 = 0xfd;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TinyEvmOpcode {
    Stop,
    Add,
    Mul,
    Sub,
    Div,
    Lt,
    Gt,
    Eq,
    IsZero,
    And,
    Or,
    Sha3,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    Pop,
    Mload,
    Mstore,
    Sload,
    Sstore,
    Jump,
    JumpI,
    JumpDest,
    Push(u8),
    Dup(u8),
    Swap(u8),
    Return,
    Revert,
}

pub fn opcode_name(byte: u8) -> Option<&'static str> {
    match byte {
        OPCODE_STOP => Some("STOP"),
        OPCODE_ADD => Some("ADD"),
        OPCODE_MUL => Some("MUL"),
        OPCODE_SUB => Some("SUB"),
        OPCODE_DIV => Some("DIV"),
        OPCODE_LT => Some("LT"),
        OPCODE_GT => Some("GT"),
        OPCODE_EQ => Some("EQ"),
        OPCODE_ISZERO => Some("ISZERO"),
        OPCODE_AND => Some("AND"),
        OPCODE_OR => Some("OR"),
        OPCODE_SHA3 => Some("SHA3"),
        OPCODE_CALLER => Some("CALLER"),
        OPCODE_CALLVALUE => Some("CALLVALUE"),
        OPCODE_CALLDATALOAD => Some("CALLDATALOAD"),
        OPCODE_CALLDATASIZE => Some("CALLDATASIZE"),
        OPCODE_POP => Some("POP"),
        OPCODE_MLOAD => Some("MLOAD"),
        OPCODE_MSTORE => Some("MSTORE"),
        OPCODE_SLOAD => Some("SLOAD"),
        OPCODE_SSTORE => Some("SSTORE"),
        OPCODE_JUMP => Some("JUMP"),
        OPCODE_JUMPI => Some("JUMPI"),
        OPCODE_JUMPDEST => Some("JUMPDEST"),
        OPCODE_PUSH1..=OPCODE_PUSH32 => Some("PUSH"),
        OPCODE_DUP1..=OPCODE_DUP16 => Some("DUP"),
        OPCODE_SWAP1..=OPCODE_SWAP16 => Some("SWAP"),
        OPCODE_RETURN => Some("RETURN"),
        OPCODE_REVERT => Some("REVERT"),
        _ => None,
    }
}

pub(crate) fn decode_opcode(byte: u8) -> Option<TinyEvmOpcode> {
    match byte {
        OPCODE_STOP => Some(TinyEvmOpcode::Stop),
        OPCODE_ADD => Some(TinyEvmOpcode::Add),
        OPCODE_MUL => Some(TinyEvmOpcode::Mul),
        OPCODE_SUB => Some(TinyEvmOpcode::Sub),
        OPCODE_DIV => Some(TinyEvmOpcode::Div),
        OPCODE_LT => Some(TinyEvmOpcode::Lt),
        OPCODE_GT => Some(TinyEvmOpcode::Gt),
        OPCODE_EQ => Some(TinyEvmOpcode::Eq),
        OPCODE_ISZERO => Some(TinyEvmOpcode::IsZero),
        OPCODE_AND => Some(TinyEvmOpcode::And),
        OPCODE_OR => Some(TinyEvmOpcode::Or),
        OPCODE_SHA3 => Some(TinyEvmOpcode::Sha3),
        OPCODE_CALLER => Some(TinyEvmOpcode::Caller),
        OPCODE_CALLVALUE => Some(TinyEvmOpcode::CallValue),
        OPCODE_CALLDATALOAD => Some(TinyEvmOpcode::CallDataLoad),
        OPCODE_CALLDATASIZE => Some(TinyEvmOpcode::CallDataSize),
        OPCODE_POP => Some(TinyEvmOpcode::Pop),
        OPCODE_MLOAD => Some(TinyEvmOpcode::Mload),
        OPCODE_MSTORE => Some(TinyEvmOpcode::Mstore),
        OPCODE_SLOAD => Some(TinyEvmOpcode::Sload),
        OPCODE_SSTORE => Some(TinyEvmOpcode::Sstore),
        OPCODE_JUMP => Some(TinyEvmOpcode::Jump),
        OPCODE_JUMPI => Some(TinyEvmOpcode::JumpI),
        OPCODE_JUMPDEST => Some(TinyEvmOpcode::JumpDest),
        OPCODE_PUSH1..=OPCODE_PUSH32 => Some(TinyEvmOpcode::Push(byte - OPCODE_PUSH1 + 1)),
        OPCODE_DUP1..=OPCODE_DUP16 => Some(TinyEvmOpcode::Dup(byte - OPCODE_DUP1 + 1)),
        OPCODE_SWAP1..=OPCODE_SWAP16 => Some(TinyEvmOpcode::Swap(byte - OPCODE_SWAP1 + 1)),
        OPCODE_RETURN => Some(TinyEvmOpcode::Return),
        OPCODE_REVERT => Some(TinyEvmOpcode::Revert),
        _ => None,
    }
}
