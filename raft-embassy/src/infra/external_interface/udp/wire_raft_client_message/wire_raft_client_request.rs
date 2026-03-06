// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::string::String;
use alloc::vec::Vec;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WireRaftClientRequest {
    Command(Vec<u8>),
    Query(String),
}

impl From<RaftClientRequest> for WireRaftClientRequest {
    fn from(request: RaftClientRequest) -> Self {
        match request {
            RaftClientRequest::Command(data) => Self::Command(data),
            RaftClientRequest::Query(query) => Self::Query(query),
        }
    }
}

impl From<WireRaftClientRequest> for RaftClientRequest {
    fn from(request: WireRaftClientRequest) -> Self {
        match request {
            WireRaftClientRequest::Command(data) => Self::Command(data),
            WireRaftClientRequest::Query(query) => Self::Query(query),
        }
    }
}
