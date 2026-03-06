// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::udp::wire_client_message::wire_transaction::WireTransaction;
use etheram_node::common_types::types::{Address, Height};
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WireClientRequest {
    GetHeight,
    GetBalance(Address),
    GetReceipts(Height),
    SubmitTransaction(WireTransaction),
}

impl From<ClientRequest> for WireClientRequest {
    fn from(request: ClientRequest) -> Self {
        match request {
            ClientRequest::GetHeight => Self::GetHeight,
            ClientRequest::GetBalance(address) => Self::GetBalance(address),
            ClientRequest::GetReceipts(height) => Self::GetReceipts(height),
            ClientRequest::SubmitTransaction(tx) => {
                Self::SubmitTransaction(WireTransaction::from(tx))
            }
        }
    }
}

impl From<WireClientRequest> for ClientRequest {
    fn from(request: WireClientRequest) -> Self {
        match request {
            WireClientRequest::GetHeight => Self::GetHeight,
            WireClientRequest::GetBalance(address) => Self::GetBalance(address),
            WireClientRequest::GetReceipts(height) => Self::GetReceipts(height),
            WireClientRequest::SubmitTransaction(tx) => Self::SubmitTransaction(tx.into()),
        }
    }
}
