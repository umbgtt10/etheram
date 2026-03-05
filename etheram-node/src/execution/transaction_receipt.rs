// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::Gas;
use crate::execution::transaction_result::TransactionStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionReceipt {
    pub status: TransactionStatus,
    pub gas_used: Gas,
    pub cumulative_gas_used: Gas,
}
