// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WireRejectionReason {
    InsufficientBalance,
    InvalidNonce,
    GasLimitExceeded,
    ZeroGasPrice,
}

impl From<TransactionRejectionReason> for WireRejectionReason {
    fn from(reason: TransactionRejectionReason) -> Self {
        match reason {
            TransactionRejectionReason::InsufficientBalance => Self::InsufficientBalance,
            TransactionRejectionReason::InvalidNonce => Self::InvalidNonce,
            TransactionRejectionReason::GasLimitExceeded => Self::GasLimitExceeded,
            TransactionRejectionReason::ZeroGasPrice => Self::ZeroGasPrice,
        }
    }
}

impl From<WireRejectionReason> for TransactionRejectionReason {
    fn from(reason: WireRejectionReason) -> Self {
        match reason {
            WireRejectionReason::InsufficientBalance => Self::InsufficientBalance,
            WireRejectionReason::InvalidNonce => Self::InvalidNonce,
            WireRejectionReason::GasLimitExceeded => Self::GasLimitExceeded,
            WireRejectionReason::ZeroGasPrice => Self::ZeroGasPrice,
        }
    }
}
