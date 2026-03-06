// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram_core::types::ClientId;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Balance;
use etheram_node::common_types::types::Gas;
use etheram_node::common_types::types::GasPrice;
use etheram_node::common_types::types::Height;
use etheram_node::common_types::types::Nonce;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
struct WireTransaction {
    from: Address,
    to: Address,
    value: Balance,
    gas_limit: Gas,
    gas_price: GasPrice,
    nonce: Nonce,
    data: Vec<u8>,
}

impl From<Transaction> for WireTransaction {
    fn from(tx: Transaction) -> Self {
        Self {
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_limit: tx.gas_limit,
            gas_price: tx.gas_price,
            nonce: tx.nonce,
            data: tx.data,
        }
    }
}

impl From<WireTransaction> for Transaction {
    fn from(wire: WireTransaction) -> Self {
        Transaction::new(
            wire.from,
            wire.to,
            wire.value,
            wire.gas_limit,
            wire.gas_price,
            wire.nonce,
            wire.data,
        )
    }
}

#[derive(Serialize, Deserialize)]
enum WireClientRequest {
    GetHeight,
    GetBalance(Address),
    GetReceipts(Height),
    SubmitTransaction(WireTransaction),
}

impl From<ClientRequest> for WireClientRequest {
    fn from(req: ClientRequest) -> Self {
        match req {
            ClientRequest::GetHeight => Self::GetHeight,
            ClientRequest::GetBalance(addr) => Self::GetBalance(addr),
            ClientRequest::GetReceipts(height) => Self::GetReceipts(height),
            ClientRequest::SubmitTransaction(tx) => {
                Self::SubmitTransaction(WireTransaction::from(tx))
            }
        }
    }
}

impl From<WireClientRequest> for ClientRequest {
    fn from(wire: WireClientRequest) -> Self {
        match wire {
            WireClientRequest::GetHeight => Self::GetHeight,
            WireClientRequest::GetBalance(addr) => Self::GetBalance(addr),
            WireClientRequest::GetReceipts(height) => Self::GetReceipts(height),
            WireClientRequest::SubmitTransaction(tx) => {
                Self::SubmitTransaction(Transaction::from(tx))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum WireRejectionReason {
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
    fn from(wire: WireRejectionReason) -> Self {
        match wire {
            WireRejectionReason::InsufficientBalance => Self::InsufficientBalance,
            WireRejectionReason::InvalidNonce => Self::InvalidNonce,
            WireRejectionReason::GasLimitExceeded => Self::GasLimitExceeded,
            WireRejectionReason::ZeroGasPrice => Self::ZeroGasPrice,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum WireClientResponse {
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
    fn from(resp: ClientResponse) -> Self {
        match resp {
            ClientResponse::Height(h) => Self::Height(h),
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
    fn from(wire: WireClientResponse) -> Self {
        match wire {
            WireClientResponse::Height(h) => Self::Height(h),
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
                reason: TransactionRejectionReason::from(reason),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WireEiRequest {
    client_id: ClientId,
    request: WireClientRequest,
}

#[derive(Serialize, Deserialize)]
struct WireEiResponse {
    client_id: ClientId,
    response: WireClientResponse,
}

pub fn serialize_ei_request(client_id: ClientId, request: ClientRequest) -> Vec<u8> {
    let wire = WireEiRequest {
        client_id,
        request: WireClientRequest::from(request),
    };
    postcard::to_allocvec(&wire).unwrap_or_default()
}

pub fn deserialize_ei_request(bytes: &[u8]) -> Option<(ClientId, ClientRequest)> {
    postcard::from_bytes::<WireEiRequest>(bytes)
        .ok()
        .map(|w| (w.client_id, ClientRequest::from(w.request)))
}

pub fn serialize_ei_response(client_id: ClientId, response: ClientResponse) -> Vec<u8> {
    let wire = WireEiResponse {
        client_id,
        response: WireClientResponse::from(response),
    };
    postcard::to_allocvec(&wire).unwrap_or_default()
}

pub fn deserialize_ei_response(bytes: &[u8]) -> Option<(ClientId, ClientResponse)> {
    postcard::from_bytes::<WireEiResponse>(bytes)
        .ok()
        .map(|w| (w.client_id, ClientResponse::from(w.response)))
}
