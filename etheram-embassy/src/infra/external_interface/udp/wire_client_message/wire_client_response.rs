// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::udp::wire_client_message::wire_rejection_reason::WireRejectionReason;
use etheram_node::common_types::types::{Balance, Height};
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WireClientResponse {
    Height(Height),
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
        reason: WireRejectionReason,
    },
}

impl From<ClientResponse> for WireClientResponse {
    fn from(response: ClientResponse) -> Self {
        match response {
            ClientResponse::Height(height) => Self::Height(height),
            ClientResponse::Balance { balance, height } => Self::Balance { balance, height },
            ClientResponse::ReceiptsSummary {
                height,
                success_count,
                out_of_gas_count,
                reverted_count,
                invalid_opcode_count,
            } => Self::ReceiptsSummary {
                height,
                success_count,
                out_of_gas_count,
                reverted_count,
                invalid_opcode_count,
            },
            ClientResponse::TransactionAccepted => Self::TransactionAccepted,
            ClientResponse::TransactionRejected { reason } => Self::TransactionRejected {
                reason: WireRejectionReason::from(reason),
            },
        }
    }
}

impl From<WireClientResponse> for ClientResponse {
    fn from(response: WireClientResponse) -> Self {
        match response {
            WireClientResponse::Height(height) => Self::Height(height),
            WireClientResponse::Balance { balance, height } => Self::Balance { balance, height },
            WireClientResponse::ReceiptsSummary {
                height,
                success_count,
                out_of_gas_count,
                reverted_count,
                invalid_opcode_count,
            } => Self::ReceiptsSummary {
                height,
                success_count,
                out_of_gas_count,
                reverted_count,
                invalid_opcode_count,
            },
            WireClientResponse::TransactionAccepted => Self::TransactionAccepted,
            WireClientResponse::TransactionRejected { reason } => Self::TransactionRejected {
                reason: reason.into(),
            },
        }
    }
}
