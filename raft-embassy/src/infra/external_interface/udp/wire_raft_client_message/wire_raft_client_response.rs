// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum WireRaftClientResponse {
    Applied(Vec<u8>),
    QueryResult(Vec<u8>),
    NotLeader(Option<u64>),
    Timeout,
}

impl From<RaftClientResponse> for WireRaftClientResponse {
    fn from(response: RaftClientResponse) -> Self {
        match response {
            RaftClientResponse::Applied(data) => Self::Applied(data),
            RaftClientResponse::QueryResult(data) => Self::QueryResult(data),
            RaftClientResponse::NotLeader(peer) => Self::NotLeader(peer),
            RaftClientResponse::Timeout => Self::Timeout,
        }
    }
}

impl From<WireRaftClientResponse> for RaftClientResponse {
    fn from(response: WireRaftClientResponse) -> Self {
        match response {
            WireRaftClientResponse::Applied(data) => Self::Applied(data),
            WireRaftClientResponse::QueryResult(data) => Self::QueryResult(data),
            WireRaftClientResponse::NotLeader(peer) => Self::NotLeader(peer),
            WireRaftClientResponse::Timeout => Self::Timeout,
        }
    }
}
