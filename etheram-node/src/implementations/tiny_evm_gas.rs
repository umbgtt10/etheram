// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Gas;

pub const INTRINSIC_GAS: Gas = 21_000;
pub const GAS_STOP: Gas = 0;
pub const GAS_ADD: Gas = 3;
pub const GAS_MUL: Gas = 5;
pub const GAS_SUB: Gas = 3;
pub const GAS_DIV: Gas = 5;
pub const GAS_LT: Gas = 3;
pub const GAS_GT: Gas = 3;
pub const GAS_EQ: Gas = 3;
pub const GAS_ISZERO: Gas = 3;
pub const GAS_AND: Gas = 3;
pub const GAS_OR: Gas = 3;
pub const GAS_SHA3_BASE: Gas = 30;
pub const GAS_SHA3_WORD: Gas = 6;
pub const GAS_CALLER: Gas = 2;
pub const GAS_CALLVALUE: Gas = 2;
pub const GAS_CALLDATALOAD: Gas = 3;
pub const GAS_CALLDATASIZE: Gas = 2;
pub const GAS_POP: Gas = 2;
pub const GAS_MLOAD_BASE: Gas = 3;
pub const GAS_MSTORE_BASE: Gas = 3;
pub const GAS_SLOAD: Gas = 800;
pub const GAS_SSTORE_SET: Gas = 20_000;
pub const GAS_SSTORE_RESET: Gas = 5_000;
pub const GAS_JUMP: Gas = 8;
pub const GAS_JUMPI: Gas = 10;
pub const GAS_JUMPDEST: Gas = 1;
pub const GAS_PUSH1: Gas = 3;
pub const GAS_PUSHN: Gas = 3;
pub const GAS_DUP: Gas = 3;
pub const GAS_SWAP: Gas = 3;
pub const GAS_RETURN: Gas = 0;
pub const GAS_REVERT: Gas = 0;

pub fn memory_expansion_cost(old_words: u64, new_words: u64) -> Gas {
    memory_cost(new_words).saturating_sub(memory_cost(old_words))
}

fn memory_cost(words: u64) -> Gas {
    3 * words + words * words / 512
}
