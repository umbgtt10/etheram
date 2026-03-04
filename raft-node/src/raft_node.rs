// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::brain::protocol::boxed_protocol::BoxedRaftProtocol;
use crate::collections::action_collection::ActionCollection;
use crate::common_types::state_machine::RaftStateMachine;
use crate::context::context_builder::RaftContextBuilder;
use crate::executor::raft_executor::RaftExecutor;
use crate::incoming::incoming_sources::RaftIncomingSources;
use crate::observer::action_kind;
use crate::observer::RaftObserver;
use crate::partitioner::partition::RaftPartitioner;
use crate::state::raft_state::RaftState;
use alloc::boxed::Box;
use etheram_core::collection::Collection;
use etheram_core::types::PeerId;

pub struct RaftNode<P: Clone + 'static> {
    peer_id: PeerId,
    incoming: RaftIncomingSources<P>,
    state: RaftState<P>,
    executor: RaftExecutor<P>,
    context_builder: Box<dyn RaftContextBuilder<P>>,
    brain: BoxedRaftProtocol<P>,
    partitioner: Box<dyn RaftPartitioner<P>>,
    state_machine: Box<dyn RaftStateMachine>,
    observer: Box<dyn RaftObserver>,
}

impl<P: Clone + AsRef<[u8]> + 'static> RaftNode<P> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        peer_id: PeerId,
        incoming: RaftIncomingSources<P>,
        state: RaftState<P>,
        executor: RaftExecutor<P>,
        context_builder: Box<dyn RaftContextBuilder<P>>,
        brain: BoxedRaftProtocol<P>,
        partitioner: Box<dyn RaftPartitioner<P>>,
        state_machine: Box<dyn RaftStateMachine>,
        observer: Box<dyn RaftObserver>,
    ) -> Self {
        let mut node = Self {
            peer_id,
            incoming,
            state,
            executor,
            context_builder,
            brain,
            partitioner,
            state_machine,
            observer,
        };
        node.observer.node_started(peer_id);
        node
    }

    pub fn state(&self) -> &RaftState<P> {
        &self.state
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn step(&mut self) -> bool {
        if let Some((source, message)) = self.incoming.poll() {
            self.observer.message_received(self.peer_id, &source);
            let context = self
                .context_builder
                .build(&self.state, self.peer_id, &source, &message);
            self.observer.context_built(
                self.peer_id,
                context.current_term,
                context.role,
                context.log.len(),
            );
            let actions = self.brain.handle_message(&source, &message, &context);
            for action in actions.iter() {
                self.observer
                    .action_emitted(self.peer_id, &action_kind(action));
            }
            let (mutations, outputs) = self.partitioner.partition(&actions);
            for mutation in mutations.iter() {
                self.observer
                    .mutation_applied(self.peer_id, &action_kind(mutation));
            }
            self.state.apply_mutations(&mutations);
            self.apply_state_machine_outputs(&outputs);
            for output in outputs.iter() {
                self.observer
                    .output_executed(self.peer_id, &action_kind(output));
            }
            self.executor.execute_outputs(&outputs);
            self.observer.step_completed(self.peer_id, true);
            return true;
        }
        self.observer.step_completed(self.peer_id, false);
        false
    }

    fn apply_state_machine_outputs(&mut self, outputs: &ActionCollection<RaftAction<P>>) {
        for action in outputs.iter() {
            if let RaftAction::ApplyToStateMachine {
                client_id: _,
                entry,
            } = action
            {
                self.state_machine
                    .apply_raw(entry.index, entry.payload.as_ref());
                self.state.set_last_applied(entry.index);
            }
            if let RaftAction::RestoreFromSnapshot(data) = action {
                self.state_machine.restore(data);
            }
        }
    }
}
