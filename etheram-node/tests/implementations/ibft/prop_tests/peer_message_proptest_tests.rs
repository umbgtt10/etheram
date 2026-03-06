// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use etheram_core::collection::Collection;
use etheram_core::consensus_protocol::ConsensusProtocol;
use etheram_node::brain::protocol::action::Action;
use etheram_node::brain::protocol::message::Message;
use etheram_node::brain::protocol::message_source::MessageSource;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::block::BLOCK_GAS_LIMIT;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Hash;
use etheram_node::context::context_dto::Context;
use etheram_node::execution::execution_engine::ExecutionEngine;
use etheram_node::execution::transaction_result::TransactionStatus;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_node::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_node::implementations::ibft::signature_scheme::SignatureBytes;
use etheram_node::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram_node::implementations::tiny_evm_engine::OPCODE_ADD;
use etheram_node::implementations::tiny_evm_engine::OPCODE_PUSH1;
use etheram_node::implementations::tiny_evm_engine::OPCODE_RETURN;
use etheram_node::implementations::tiny_evm_engine::OPCODE_STOP;
use proptest::prelude::*;

fn arb_context() -> BoxedStrategy<Context> {
    (0u64..4u64, 0u64..10u64)
        .prop_map(|(peer_id, height)| Context::new(peer_id, height, [0u8; 32]))
        .boxed()
}

fn arb_hash() -> BoxedStrategy<Hash> {
    any::<[u8; 32]>().boxed()
}

fn arb_simple_bytecode() -> BoxedStrategy<alloc::vec::Vec<u8>> {
    let opcodes = prop::collection::vec(
        prop_oneof![
            Just(OPCODE_STOP),
            (any::<u8>())
                .prop_map(|b| vec![OPCODE_PUSH1, b])
                .prop_map(|v| v[0]),
            Just(OPCODE_ADD),
            Just(OPCODE_RETURN),
        ],
        1..=16,
    );
    opcodes
        .prop_flat_map(|ops| {
            let mut bytecode = alloc::vec::Vec::new();
            for op in ops {
                if op == OPCODE_PUSH1 {
                    bytecode.push(OPCODE_PUSH1);
                    bytecode.push(0x01);
                } else {
                    bytecode.push(op);
                }
            }
            Just(bytecode)
        })
        .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prepare_for_future_round_always_returns_empty(
        ctx in arb_context(),
        sender in 0u64..4u64,
        future_round in 1u64..=10u64,
        block_hash in arb_hash(),
    ) {
        // Arrange
        let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(ctx.peer_id)));
        let message = Message::Peer(IbftMessage::Prepare {
            sequence: 0,
            height: ctx.current_height,
            round: future_round,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        });

        // Act
        let actions = protocol.handle_message(&MessageSource::Peer(sender), &message, &ctx);

        // Assert
        prop_assert_eq!(actions.len(), 0);
    }

    #[test]
    fn commit_for_future_round_always_returns_empty(
        ctx in arb_context(),
        sender in 0u64..4u64,
        future_round in 1u64..=10u64,
        block_hash in arb_hash(),
    ) {
        // Arrange
        let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(ctx.peer_id)));
        let message = Message::Peer(IbftMessage::Commit {
            sequence: 0,
            height: ctx.current_height,
            round: future_round,
            block_hash,
            sender_signature: SignatureBytes::zeroed(),
        });

        // Act
        let actions = protocol.handle_message(&MessageSource::Peer(sender), &message, &ctx);

        // Assert
        prop_assert_eq!(actions.len(), 0);
    }

    #[test]
    fn single_view_change_never_produces_new_view(
        ctx in arb_context(),
        sender in 0u64..4u64,
        vc_round in 0u64..=5u64,
    ) {
        // Arrange
        let mut protocol = IbftProtocol::new(vec![0, 1, 2, 3], Box::new(MockSignatureScheme::new(ctx.peer_id)));
        let message = Message::Peer(IbftMessage::ViewChange {
            sequence: 0,
            height: ctx.current_height,
            round: vc_round,
            prepared_certificate: None,
        });

        // Act
        let actions = protocol.handle_message(&MessageSource::Peer(sender), &message, &ctx);

        // Assert
        let has_new_view = actions.iter().any(|a| matches!(
            a,
            Action::BroadcastMessage { message: IbftMessage::NewView { .. } }
        ));
        prop_assert!(!has_new_view);
    }

    #[test]
    fn execute_bytecode_never_panics(bytecode in arb_simple_bytecode()) {
        // Arrange
        let sender = [0xaau8; 20];
        let contract = [0xbbu8; 20];
        let transaction = Transaction::new(sender, contract, 0, 100_000, 1, 0, bytecode);
        let block = Block::new(0, 0, vec![transaction], [0u8; 32], BLOCK_GAS_LIMIT);
        let accounts = BTreeMap::from([
            (sender, Account::new(1_000_000)),
            (contract, Account::new(0)),
        ]);

        // Act
        let result = TinyEvmEngine.execute(&block, &accounts, &BTreeMap::new());

        // Assert
        let status = &result.transaction_results[0].status;
        prop_assert!(
            matches!(status, TransactionStatus::Success) || matches!(status, TransactionStatus::OutOfGas)
        );
    }
}
