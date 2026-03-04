// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::builders::raft_cache_builder::RaftCacheBuilder;
use crate::builders::raft_context_builder_builder::RaftContextBuilderBuilder;
use crate::builders::raft_observer_builder::RaftObserverBuilder;
use crate::builders::raft_partitioner_builder::RaftPartitionerBuilder;
use crate::builders::raft_protocol_builder::RaftProtocolBuilder;
use crate::builders::raft_state_machine_builder::RaftStateMachineBuilder;
use crate::builders::raft_storage_builder::RaftStorageBuilder;
use crate::implementations::eager_raft_context_builder::EagerRaftContextBuilder;
use crate::implementations::in_memory_raft_cache::InMemoryRaftCache;
use crate::implementations::in_memory_raft_state_machine::InMemoryRaftStateMachine;
use crate::implementations::in_memory_raft_storage::InMemoryRaftStorage;
use crate::implementations::no_op_raft_observer::NoOpRaftObserver;
use crate::implementations::no_op_raft_transport::NoOpRaftTransport;
use crate::implementations::raft::raft_protocol::RaftProtocol;
use crate::implementations::type_based_raft_partitioner::TypeBasedRaftPartitioner;
use crate::variants::{
    RaftCacheVariant, RaftContextBuilderVariant, RaftObserverVariant, RaftPartitionerVariant,
    RaftProtocolVariant, RaftStateMachineVariant, RaftStorageVariant,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::boxed_protocol::BoxedRaftProtocol;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::cache_adapter::CacheAdapter;
use raft_node::common_types::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use raft_node::common_types::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use raft_node::common_types::state_machine::RaftStateMachine;
use raft_node::common_types::storage_adapter::StorageAdapter;
use raft_node::common_types::timer_input_adapter::TimerInputAdapter;
use raft_node::common_types::timer_output_adapter::TimerOutputAdapter;
use raft_node::common_types::transport_incoming_adapter::TransportIncomingAdapter;
use raft_node::common_types::transport_outgoing_adapter::TransportOutgoingAdapter;
use raft_node::context::context_builder::RaftContextBuilder;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::executor::outgoing::outgoing_sources::RaftOutgoingSources;
use raft_node::executor::raft_executor::RaftExecutor;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use raft_node::incoming::incoming_sources::RaftIncomingSources;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_node::observer::RaftObserver;
use raft_node::partitioner::partition::RaftPartitioner;
use raft_node::raft_node::RaftNode;
use raft_node::state::raft_state::RaftState;

pub struct RaftNodeBuilder<P: Clone + 'static> {
    peer_id: Option<PeerId>,
    peers: Vec<PeerId>,
    timer_input: Option<Box<dyn TimerInputAdapter<RaftTimerEvent>>>,
    timer_output: Option<Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>>,
    transport_incoming: Option<Box<dyn TransportIncomingAdapter<RaftMessage<P>>>>,
    transport_outgoing: Option<Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>>,
    ei_incoming: Option<Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>>,
    ei_outgoing: Option<Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>>,
    storage: Option<Box<dyn StorageAdapter<P, Key = (), Value = ()>>>,
    cache: Option<Box<dyn CacheAdapter<Key = (), Value = ()>>>,
    context_builder: Option<Box<dyn RaftContextBuilder<P>>>,
    brain: Option<BoxedRaftProtocol<P>>,
    partitioner: Option<Box<dyn RaftPartitioner<P>>>,
    state_machine: Option<Box<dyn RaftStateMachine>>,
    observer: Option<Box<dyn RaftObserver>>,
}

impl<P: Clone + 'static> RaftNodeBuilder<P> {
    pub fn new() -> Self {
        Self {
            peer_id: None,
            peers: Vec::new(),
            timer_input: None,
            timer_output: None,
            transport_incoming: None,
            transport_outgoing: None,
            ei_incoming: None,
            ei_outgoing: None,
            storage: None,
            cache: None,
            context_builder: None,
            brain: None,
            partitioner: None,
            state_machine: None,
            observer: None,
        }
    }

    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn with_peers(mut self, peers: Vec<PeerId>) -> Self {
        self.peers = peers;
        self
    }

    pub fn with_timer_input(mut self, t: Box<dyn TimerInputAdapter<RaftTimerEvent>>) -> Self {
        self.timer_input = Some(t);
        self
    }

    pub fn with_timer_output(
        mut self,
        t: Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>,
    ) -> Self {
        self.timer_output = Some(t);
        self
    }

    pub fn with_transport_incoming(
        mut self,
        t: Box<dyn TransportIncomingAdapter<RaftMessage<P>>>,
    ) -> Self {
        self.transport_incoming = Some(t);
        self
    }

    pub fn with_transport_outgoing(
        mut self,
        t: Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>,
    ) -> Self {
        self.transport_outgoing = Some(t);
        self
    }

    pub fn with_ei_incoming(
        mut self,
        ei: Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>,
    ) -> Self {
        self.ei_incoming = Some(ei);
        self
    }

    pub fn with_ei_outgoing(
        mut self,
        ei: Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>,
    ) -> Self {
        self.ei_outgoing = Some(ei);
        self
    }

    pub fn with_storage(mut self, s: Box<dyn StorageAdapter<P, Key = (), Value = ()>>) -> Self {
        self.storage = Some(s);
        self
    }

    pub fn with_storage_variant(mut self, variant: RaftStorageVariant<P>) -> Self {
        self.storage = Some(
            RaftStorageBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    pub fn with_cache(mut self, c: Box<dyn CacheAdapter<Key = (), Value = ()>>) -> Self {
        self.cache = Some(c);
        self
    }

    pub fn with_cache_variant(mut self, variant: RaftCacheVariant) -> Self {
        self.cache = Some(
            RaftCacheBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    pub fn with_brain(mut self, brain: BoxedRaftProtocol<P>) -> Self {
        self.brain = Some(brain);
        self
    }

    pub fn with_brain_variant(mut self, variant: RaftProtocolVariant<P>) -> Self
    where
        P: From<Vec<u8>> + AsRef<[u8]>,
    {
        self.brain = Some(
            RaftProtocolBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    pub fn with_state_machine(mut self, sm: Box<dyn RaftStateMachine>) -> Self {
        self.state_machine = Some(sm);
        self
    }

    pub fn with_state_machine_variant(mut self, variant: RaftStateMachineVariant) -> Self {
        self.state_machine = Some(
            RaftStateMachineBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    pub fn with_context_builder(mut self, cb: Box<dyn RaftContextBuilder<P>>) -> Self {
        self.context_builder = Some(cb);
        self
    }

    pub fn with_context_builder_variant(mut self, variant: RaftContextBuilderVariant<P>) -> Self {
        self.context_builder = Some(
            RaftContextBuilderBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    pub fn with_partitioner(mut self, p: Box<dyn RaftPartitioner<P>>) -> Self {
        self.partitioner = Some(p);
        self
    }

    pub fn with_partitioner_variant(mut self, variant: RaftPartitionerVariant<P>) -> Self {
        self.partitioner = Some(
            RaftPartitionerBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    pub fn with_observer(mut self, obs: Box<dyn RaftObserver>) -> Self {
        self.observer = Some(obs);
        self
    }

    pub fn with_observer_variant(mut self, variant: RaftObserverVariant) -> Self {
        self.observer = Some(
            RaftObserverBuilder::new()
                .with_variant(variant)
                .build()
                .unwrap(),
        );
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build(self) -> Result<RaftNode<P>, BuildError>
    where
        P: From<Vec<u8>> + AsRef<[u8]>,
    {
        let peer_id = self
            .peer_id
            .ok_or(BuildError::MissingComponent("peer_id"))?;
        let peers = self.peers;

        let timer_in: Box<dyn TimerInputAdapter<RaftTimerEvent>> = self
            .timer_input
            .ok_or(BuildError::MissingComponent("timer_input"))?;
        let timer_out: Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>> = self
            .timer_output
            .ok_or(BuildError::MissingComponent("timer_output"))?;
        let transport_in: Box<dyn TransportIncomingAdapter<RaftMessage<P>>> = self
            .transport_incoming
            .unwrap_or_else(|| Box::new(NoOpRaftTransport::<P>::new()));
        let transport_out: Box<dyn TransportOutgoingAdapter<RaftMessage<P>>> = self
            .transport_outgoing
            .unwrap_or_else(|| Box::new(NoOpRaftTransport::<P>::new()));
        let ei_in: Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>> = self
            .ei_incoming
            .ok_or(BuildError::MissingComponent("ei_incoming"))?;
        let ei_out: Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>> = self
            .ei_outgoing
            .ok_or(BuildError::MissingComponent("ei_outgoing"))?;

        let storage: Box<dyn StorageAdapter<P, Key = (), Value = ()>> = self
            .storage
            .unwrap_or_else(|| Box::new(InMemoryRaftStorage::new()));
        let cache: Box<dyn CacheAdapter<Key = (), Value = ()>> = self
            .cache
            .unwrap_or_else(|| Box::new(InMemoryRaftCache::new()));

        let context_builder: Box<dyn RaftContextBuilder<P>> = self
            .context_builder
            .unwrap_or_else(|| Box::new(EagerRaftContextBuilder::new(peer_id, peers.clone())));

        let brain: BoxedRaftProtocol<P> = self
            .brain
            .unwrap_or_else(|| Box::new(RaftProtocol::<P>::new()));

        let partitioner: Box<dyn RaftPartitioner<P>> = self
            .partitioner
            .unwrap_or_else(|| Box::new(TypeBasedRaftPartitioner::new()));

        let state_machine: Box<dyn RaftStateMachine> = self
            .state_machine
            .unwrap_or_else(|| Box::new(InMemoryRaftStateMachine::new()));

        let observer: Box<dyn RaftObserver> = self
            .observer
            .unwrap_or_else(|| Box::new(NoOpRaftObserver::new()));

        let incoming = RaftIncomingSources::new(timer_in, ei_in, transport_in);
        let outgoing = RaftOutgoingSources::new(timer_out, ei_out, transport_out);
        let executor = RaftExecutor::new_with_peers(outgoing, peers.clone());
        let state = RaftState::new(storage, cache);

        Ok(RaftNode::new(
            peer_id,
            incoming,
            state,
            executor,
            context_builder,
            brain,
            partitioner,
            state_machine,
            observer,
        ))
    }
}

impl<P: Clone + 'static> Default for RaftNodeBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}
