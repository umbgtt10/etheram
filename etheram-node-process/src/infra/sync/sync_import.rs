// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::wire_ibft_message::deserialize_block;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::types::Hash;

pub fn decode_and_validate_blocks(
    local_height: u64,
    start_height: u64,
    block_payloads: &[Vec<u8>],
    expected_parent_post_state_root: Option<Hash>,
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

        if block.gas_limit == 0 || block.gas_limit > BLOCK_GAS_LIMIT {
            return None;
        }

        let total_tx_gas = block
            .transactions
            .iter()
            .try_fold(0u64, |acc, tx| acc.checked_add(tx.gas_limit))?;
        if total_tx_gas > block.gas_limit {
            return None;
        }

        if offset == 0 {
            if let Some(parent_post_state_root) = expected_parent_post_state_root {
                if block.state_root != parent_post_state_root {
                    return None;
                }
            }
        }

        blocks.push(block);
    }

    Some(blocks)
}
