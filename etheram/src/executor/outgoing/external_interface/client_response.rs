// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::types::{Balance, Height};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientResponse {
    Balance { balance: Balance, height: Height },
    TransactionAccepted,
    TransactionRejected { reason: TransactionRejectionReason },
    Height(Height),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionRejectionReason {
    InsufficientBalance,
    InvalidNonce,
    GasLimitExceeded,
}
