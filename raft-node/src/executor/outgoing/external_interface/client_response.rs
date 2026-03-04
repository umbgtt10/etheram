// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram_core::types::PeerId;

#[derive(Debug, Clone)]
pub enum RaftClientResponse {
    Applied(Vec<u8>),
    QueryResult(Vec<u8>),
    NotLeader(Option<PeerId>),
    Timeout,
}
