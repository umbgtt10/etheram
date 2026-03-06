// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::transaction::Transaction;
use etheram_node_process::infra::sync::sync_import::decode_and_validate_blocks;
use etheram_node_process::infra::transport::grpc_transport::wire_ibft_message::serialize_block;
use proptest::prelude::*;

fn arb_transaction(max_gas: u64) -> BoxedStrategy<Transaction> {
    let gas_bound = max_gas.min(100_000);
    (
        any::<[u8; 20]>(),
        any::<[u8; 20]>(),
        0u128..=1_000_000u128,
        1u64..=gas_bound,
        0u64..=100u64,
        0u64..=10u64,
    )
        .prop_map(
            |(from, to, value, gas_limit, gas_price, nonce)| Transaction {
                from,
                to,
                value,
                gas_limit,
                gas_price,
                nonce,
                data: Vec::new(),
            },
        )
        .boxed()
}

fn arb_block(height: u64) -> BoxedStrategy<Block> {
    let gas_limit = 1u64..=BLOCK_GAS_LIMIT;
    (any::<u64>(), any::<[u8; 32]>(), gas_limit)
        .prop_flat_map(move |(proposer, state_root, gas_limit)| {
            let tx_count = 0usize..=3;
            let max_per_tx = gas_limit / 4;
            (
                Just(proposer),
                Just(state_root),
                Just(gas_limit),
                proptest::collection::vec(arb_transaction(max_per_tx), tx_count),
            )
        })
        .prop_map(
            move |(proposer, state_root, gas_limit, transactions)| Block {
                height,
                proposer,
                transactions,
                state_root,
                post_state_root: [0u8; 32],
                receipts_root: [0u8; 32],
                gas_limit,
            },
        )
        .boxed()
}

fn serialize_blocks(blocks: &[Block]) -> Vec<Vec<u8>> {
    blocks
        .iter()
        .map(|block| serialize_block(block).expect("serialize_block failed"))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn valid_contiguous_blocks_always_decode_successfully(
        start in 0u64..100u64,
        count in 1usize..=4,
    ) {
        // Arrange
        let blocks: Vec<Block> = (0..count)
            .map(|offset| {
                Block::empty(start + offset as u64, 1, [offset as u8; 32])
            })
            .collect();
        let payloads = serialize_blocks(&blocks);

        // Act
        let result = decode_and_validate_blocks(start, start, &payloads, None);

        // Assert
        prop_assert!(result.is_some());
        prop_assert_eq!(result.unwrap().len(), count);
    }

    #[test]
    fn height_mismatch_always_returns_none(
        local in 0u64..100u64,
        start in 0u64..100u64,
    ) {
        // Arrange
        prop_assume!(local != start);
        let block = Block::empty(start, 1, [0u8; 32]);
        let payloads = serialize_blocks(&[block]);

        // Act
        let result = decode_and_validate_blocks(local, start, &payloads, None);

        // Assert
        prop_assert!(result.is_none());
    }

    #[test]
    fn zero_gas_limit_block_always_rejected(
        height in 0u64..100u64,
        proposer in any::<u64>(),
    ) {
        // Arrange
        let block = Block {
            height,
            proposer,
            transactions: Vec::new(),
            state_root: [0u8; 32],
            post_state_root: [0u8; 32],
            receipts_root: [0u8; 32],
            gas_limit: 0,
        };
        let payloads = serialize_blocks(&[block]);

        // Act
        let result = decode_and_validate_blocks(height, height, &payloads, None);

        // Assert
        prop_assert!(result.is_none());
    }

    #[test]
    fn over_limit_gas_always_rejected(
        height in 0u64..100u64,
        excess in 1u64..1_000_000u64,
    ) {
        // Arrange
        let block = Block {
            height,
            proposer: 1,
            transactions: Vec::new(),
            state_root: [0u8; 32],
            post_state_root: [0u8; 32],
            receipts_root: [0u8; 32],
            gas_limit: BLOCK_GAS_LIMIT + excess,
        };
        let payloads = serialize_blocks(&[block]);

        // Act
        let result = decode_and_validate_blocks(height, height, &payloads, None);

        // Assert
        prop_assert!(result.is_none());
    }

    #[test]
    fn state_root_mismatch_at_first_block_always_rejected(
        height in 0u64..100u64,
        root_a in any::<[u8; 32]>(),
        root_b in any::<[u8; 32]>(),
    ) {
        // Arrange
        prop_assume!(root_a != root_b);
        let block = Block::empty(height, 1, root_a);
        let payloads = serialize_blocks(&[block]);

        // Act
        let result = decode_and_validate_blocks(height, height, &payloads, Some(root_b));

        // Assert
        prop_assert!(result.is_none());
    }

    #[test]
    fn matching_state_root_at_first_block_accepted(
        height in 0u64..100u64,
        root in any::<[u8; 32]>(),
    ) {
        // Arrange
        let block = Block::empty(height, 1, root);
        let payloads = serialize_blocks(&[block]);

        // Act
        let result = decode_and_validate_blocks(height, height, &payloads, Some(root));

        // Assert
        prop_assert!(result.is_some());
    }

    #[test]
    fn round_trip_block_preserves_all_fields(block in arb_block(5)) {
        // Arrange
        let payloads = serialize_blocks(std::slice::from_ref(&block));

        // Act
        let result = decode_and_validate_blocks(5, 5, &payloads, None);

        // Assert
        prop_assert!(result.is_some());
        let decoded = result.unwrap();
        prop_assert_eq!(decoded.len(), 1);
        prop_assert_eq!(&decoded[0], &block);
    }
}
