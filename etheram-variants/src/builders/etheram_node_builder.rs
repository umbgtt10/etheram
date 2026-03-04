// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::value_transfer_engine::ValueTransferEngine;
use alloc::boxed::Box;
use etheram::brain::protocol::boxed_protocol::BoxedProtocol;
use etheram::common_types::account::Account;
use etheram::common_types::cache_adapter::CacheAdapter;
use etheram::common_types::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use etheram::common_types::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use etheram::common_types::storage_adapter::StorageAdapter;
use etheram::common_types::timer_input_adapter::TimerInputAdapter;
use etheram::common_types::timer_output_adapter::TimerOutputAdapter;
use etheram::common_types::transaction::Transaction;
use etheram::common_types::transport_incoming_adapter::TransportInputAdapter;
use etheram::common_types::transport_outgoing_adapter::TransportOutputAdapter;
use etheram::common_types::types::Address;
use etheram::context::context_builder::ContextBuilder;
use etheram::etheram_node::EtheramNode;
use etheram::execution::execution_engine::BoxedExecutionEngine;
use etheram::executor::etheram_executor::EtheramExecutor;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::incoming_sources::IncomingSources;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram::observer::Observer;
use etheram::partitioner::partition::Partitioner;
use etheram::state::etheram_state::EtheramState;
use etheram_core::types::PeerId;

pub struct EtheramNodeBuilder {
    peer_id: Option<PeerId>,

    timer_input: Option<Box<dyn TimerInputAdapter<TimerEvent>>>,
    timer_output: Option<Box<dyn TimerOutputAdapter<TimerEvent, u64>>>,
    transport_incoming: Option<Box<dyn TransportInputAdapter<()>>>,
    transport_outgoing: Option<Box<dyn TransportOutputAdapter<()>>>,
    external_interface_incoming: Option<Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>>,
    external_interface_outgoing: Option<Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>>,
    storage: Option<Box<dyn StorageAdapter<Key = Address, Value = Account>>>,
    cache: Option<Box<dyn CacheAdapter<Key = (), Value = Transaction>>>,

    context_builder: Option<Box<dyn ContextBuilder<()>>>,
    brain: Option<BoxedProtocol<()>>,
    partitioner: Option<Box<dyn Partitioner<()>>>,
    execution_engine: BoxedExecutionEngine,
    observer: Option<Box<dyn Observer>>,
}

impl EtheramNodeBuilder {
    pub fn new() -> Self {
        Self {
            peer_id: None,
            timer_input: None,
            timer_output: None,
            transport_incoming: None,
            transport_outgoing: None,
            external_interface_incoming: None,
            external_interface_outgoing: None,
            storage: None,
            cache: None,
            context_builder: None,
            brain: None,
            partitioner: None,
            execution_engine: Box::new(ValueTransferEngine),
            observer: None,
        }
    }

    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn with_timer_input(mut self, timer: Box<dyn TimerInputAdapter<TimerEvent>>) -> Self {
        self.timer_input = Some(timer);
        self
    }

    pub fn with_timer_output(
        mut self,
        timer: Box<dyn TimerOutputAdapter<TimerEvent, u64>>,
    ) -> Self {
        self.timer_output = Some(timer);
        self
    }

    pub fn with_transport_incoming(
        mut self,
        transport: Box<dyn TransportInputAdapter<()>>,
    ) -> Self {
        self.transport_incoming = Some(transport);
        self
    }

    pub fn with_transport_outgoing(
        mut self,
        transport: Box<dyn TransportOutputAdapter<()>>,
    ) -> Self {
        self.transport_outgoing = Some(transport);
        self
    }

    pub fn with_external_interface_incoming(
        mut self,
        external_interface: Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>,
    ) -> Self {
        self.external_interface_incoming = Some(external_interface);
        self
    }

    pub fn with_external_interface_outgoing(
        mut self,
        external_interface: Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>,
    ) -> Self {
        self.external_interface_outgoing = Some(external_interface);
        self
    }

    pub fn with_storage(
        mut self,
        storage: Box<dyn StorageAdapter<Key = Address, Value = Account>>,
    ) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_cache(
        mut self,
        cache: Box<dyn CacheAdapter<Key = (), Value = Transaction>>,
    ) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn with_protocol(mut self, protocol: BoxedProtocol<()>) -> Self {
        self.brain = Some(protocol);
        self
    }

    pub fn with_context_builder(mut self, context_builder: Box<dyn ContextBuilder<()>>) -> Self {
        self.context_builder = Some(context_builder);
        self
    }

    pub fn with_partitioner(mut self, partitioner: Box<dyn Partitioner<()>>) -> Self {
        self.partitioner = Some(partitioner);
        self
    }

    pub fn with_execution_engine(mut self, execution_engine: BoxedExecutionEngine) -> Self {
        self.execution_engine = execution_engine;
        self
    }

    pub fn with_observer(mut self, observer: Box<dyn Observer>) -> Self {
        self.observer = Some(observer);
        self
    }

    pub fn with_scheduler(
        mut self,
        scheduler: (Box<dyn ContextBuilder<()>>, Box<dyn Partitioner<()>>),
    ) -> Self {
        self.context_builder = Some(scheduler.0);
        self.partitioner = Some(scheduler.1);
        self
    }

    pub fn build(self) -> Result<EtheramNode<()>, BuildError> {
        let peer_id = self
            .peer_id
            .ok_or(BuildError::MissingComponent("peer_id"))?;

        let timer_input = self
            .timer_input
            .ok_or(BuildError::MissingComponent("timer_input"))?;
        let timer_output = self
            .timer_output
            .ok_or(BuildError::MissingComponent("timer_output"))?;
        let transport_incoming = self
            .transport_incoming
            .ok_or(BuildError::MissingComponent("transport_incoming"))?;
        let transport_outgoing = self
            .transport_outgoing
            .ok_or(BuildError::MissingComponent("transport_outgoing"))?;
        let external_interface_incoming = self
            .external_interface_incoming
            .ok_or(BuildError::MissingComponent("external_interface_incoming"))?;
        let external_interface_outgoing = self
            .external_interface_outgoing
            .ok_or(BuildError::MissingComponent("external_interface_outgoing"))?;
        let storage = self
            .storage
            .ok_or(BuildError::MissingComponent("storage"))?;
        let cache = self.cache.ok_or(BuildError::MissingComponent("cache"))?;

        let context_builder = self
            .context_builder
            .ok_or(BuildError::MissingComponent("context_builder"))?;
        let brain = self.brain.ok_or(BuildError::MissingComponent("brain"))?;
        let partitioner = self
            .partitioner
            .ok_or(BuildError::MissingComponent("partitioner"))?;
        let execution_engine = self.execution_engine;
        let observer = self
            .observer
            .ok_or(BuildError::MissingComponent("observer"))?;

        let incoming =
            IncomingSources::new(timer_input, external_interface_incoming, transport_incoming);
        let state = EtheramState::new(storage, cache);
        let outgoing = OutgoingSources::new(
            timer_output,
            external_interface_outgoing,
            transport_outgoing,
        );
        let executor = EtheramExecutor::new(outgoing);

        Ok(EtheramNode::new(
            peer_id,
            incoming,
            state,
            executor,
            context_builder,
            brain,
            partitioner,
            execution_engine,
            observer,
        ))
    }
}

impl Default for EtheramNodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
