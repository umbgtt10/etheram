// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::wire_ibft_message::deserialize_block;
use etheram_node::common_types::block::Block;

pub fn decode_and_validate_blocks(
    local_height: u64,
    start_height: u64,
    block_payloads: &[Vec<u8>],
) -> Option<Vec<Block>> {
    if start_height != local_height {
        return None;
    }

    let mut blocks = Vec::new();
    for (offset, payload) in block_payloads.iter().enumerate() {
        let block = deserialize_block(payload).ok()?;
        let expected_height = start_height + offset as u64;
        if block.height != expected_height {
            return None;
        }
        blocks.push(block);
    }

    Some(blocks)
}
