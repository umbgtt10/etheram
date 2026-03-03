// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram::common_types::types::Gas;

pub const INTRINSIC_GAS: Gas = 21_000;
pub const GAS_STOP: Gas = 0;
pub const GAS_ADD: Gas = 3;
pub const GAS_SLOAD: Gas = 800;
pub const GAS_SSTORE_SET: Gas = 20_000;
pub const GAS_SSTORE_RESET: Gas = 5_000;
pub const GAS_PUSH1: Gas = 3;
pub const GAS_RETURN: Gas = 0;
