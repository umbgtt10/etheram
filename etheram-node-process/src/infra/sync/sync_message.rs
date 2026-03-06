// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use serde::Deserialize;
use serde::Serialize;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMessage {
    Status {
        height: Height,
        last_hash: Hash,
    },
    GetBlocks {
        from_height: Height,
        max_blocks: u64,
    },
    Blocks {
        start_height: Height,
        block_payloads: Vec<Vec<u8>>,
    },
}
