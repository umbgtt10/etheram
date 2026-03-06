// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{
    common_types::types::{Balance, Height},
    executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientResponse {
    Balance {
        balance: Balance,
        height: Height,
    },
    ReceiptsSummary {
        height: Height,
        success_count: u64,
        out_of_gas_count: u64,
        reverted_count: u64,
        invalid_opcode_count: u64,
    },
    TransactionAccepted,
    TransactionRejected {
        reason: TransactionRejectionReason,
    },
    Height(Height),
}
