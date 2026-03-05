// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::Action;
use crate::brain::protocol::boxed_protocol::BoxedProtocol;
use crate::common_types::block::Block;
use crate::context::context_builder::ContextBuilder;
use crate::execution::execution_engine::BoxedExecutionEngine;
use crate::execution::transaction_receipt::TransactionReceipt;
use crate::execution::transaction_result::TransactionStatus;
use crate::executor::etheram_executor::EtheramExecutor;
use crate::incoming::incoming_sources::IncomingSources;
use crate::observer::action_kind;
use crate::observer::ActionKind;
use crate::observer::Observer;
use crate::partitioner::partition::Partitioner;
use crate::state::etheram_state::EtheramState;
use crate::state::storage::storage_mutation::StorageMutation;
use alloc::boxed::Box;
use alloc::vec::Vec;
use etheram_core::collection::Collection;
use etheram_core::types::PeerId;

pub struct EtheramNode<M: Clone + 'static> {
    peer_id: PeerId,
    incoming: IncomingSources<M>,
    state: EtheramState,
    executor: EtheramExecutor<M>,
    context_builder: Box<dyn ContextBuilder<M>>,
    brain: BoxedProtocol<M>,
    partitioner: Box<dyn Partitioner<M>>,
    execution_engine: BoxedExecutionEngine,
    observer: Box<dyn Observer>,
}

impl<M: Clone + 'static> EtheramNode<M> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        peer_id: PeerId,
        incoming: IncomingSources<M>,
        state: EtheramState,
        executor: EtheramExecutor<M>,
        context_builder: Box<dyn ContextBuilder<M>>,
        brain: BoxedProtocol<M>,
        partitioner: Box<dyn Partitioner<M>>,
        execution_engine: BoxedExecutionEngine,
        observer: Box<dyn Observer>,
    ) -> Self {
        let mut node = Self {
            peer_id,
            incoming,
            state,
            executor,
            context_builder,
            brain,
            partitioner,
            execution_engine,
            observer,
        };
        node.observer.node_started(peer_id);
        node
    }
    pub fn step(&mut self) -> bool {
        if let Some((source, message)) = self.incoming.poll() {
            self.observer.message_received(self.peer_id, &source);
            let context = self
                .context_builder
                .build(&self.state, self.peer_id, &source, &message);
            self.observer.context_built(
                self.peer_id,
                context.current_height,
                context.state_root,
                context.pending_txs.len(),
            );
            let actions = self.brain.handle_message(&source, &message, &context);
            for action in actions.iter() {
                self.observer
                    .action_emitted(self.peer_id, &action_kind(action));
            }
            let (mutations, outputs, executions) = self.partitioner.partition(&actions);
            for mutation in mutations.iter() {
                self.observer
                    .mutation_applied(self.peer_id, &action_kind(mutation));
            }
            self.state.apply_mutations(&mutations);
            for output in outputs.iter() {
                self.observer
                    .output_executed(self.peer_id, &action_kind(output));
            }
            self.executor.execute_outputs(&outputs);
            for execution in executions.iter() {
                if let Action::ExecuteBlock { block } = execution {
                    self.execute_block(block);
                }
            }
            self.observer.step_completed(self.peer_id, true);
            return true;
        }
        self.observer.step_completed(self.peer_id, false);
        false
    }

    fn execute_block(&mut self, block: &Block) {
        let accounts = self.state.snapshot_accounts();
        let contract_storage = self.state.snapshot_contract_storage();
        let execution_result = self
            .execution_engine
            .execute(block, &accounts, &contract_storage);
        let block_height = block.height;
        let mut cumulative_gas_used: u64 = 0;
        let mut receipts = Vec::new();
        for tx_result in execution_result.transaction_results {
            cumulative_gas_used += tx_result.gas_used;
            receipts.push(TransactionReceipt {
                status: tx_result.status,
                gas_used: tx_result.gas_used,
                cumulative_gas_used,
            });
            match tx_result.status {
                TransactionStatus::Success => {
                    for mutation in tx_result.mutations {
                        let mutation_kind = storage_mutation_kind(&mutation);
                        self.observer.mutation_applied(self.peer_id, &mutation_kind);
                        self.state.apply_single_mutation(mutation);
                    }
                }
                TransactionStatus::OutOfGas => {
                    self.observer.mutation_applied(
                        self.peer_id,
                        &ActionKind::TransactionReverted {
                            address: tx_result.from,
                        },
                    );
                }
            }
        }
        let (success_count, out_of_gas_count) =
            receipts
                .iter()
                .fold((0usize, 0usize), |(s, o), r| match r.status {
                    TransactionStatus::Success => (s + 1, o),
                    TransactionStatus::OutOfGas => (s, o + 1),
                });
        self.observer.mutation_applied(
            self.peer_id,
            &ActionKind::StoreReceipts {
                height: block_height,
                success_count,
                out_of_gas_count,
            },
        );
        self.state
            .apply_single_mutation(StorageMutation::StoreReceipts(block_height, receipts));
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn state(&self) -> &EtheramState {
        &self.state
    }
}

fn storage_mutation_kind(mutation: &StorageMutation) -> ActionKind {
    match mutation {
        StorageMutation::UpdateAccount(address, _) => {
            ActionKind::UpdateAccount { address: *address }
        }
        StorageMutation::UpdateContractStorage { address, .. } => {
            ActionKind::UpdateContractStorage { address: *address }
        }
        StorageMutation::IncrementHeight => ActionKind::IncrementHeight,
        StorageMutation::StoreBlock(block) => ActionKind::StoreBlock {
            height: block.height,
        },
        StorageMutation::StoreReceipts(height, receipts) => {
            let (s, o) = receipts
                .iter()
                .fold((0usize, 0usize), |(s, o), r| match r.status {
                    TransactionStatus::Success => (s + 1, o),
                    TransactionStatus::OutOfGas => (s, o + 1),
                });
            ActionKind::StoreReceipts {
                height: *height,
                success_count: s,
                out_of_gas_count: o,
            }
        }
    }
}
