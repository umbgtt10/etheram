// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use etheram_core::node_common::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use etheram_core::node_common::timer_input_adapter::TimerInputAdapter;
use etheram_core::node_common::timer_output_adapter::TimerOutputAdapter;
use etheram_core::node_common::transport_incoming_adapter::TransportIncomingAdapter;
use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;
use etheram_core::types::PeerId;
use etheram_node::brain::protocol::boxed_protocol::BoxedProtocol;
use etheram_node::builders::cache_builder::CacheBuilder;
use etheram_node::builders::context_builder_builder::ContextBuilderBuilder;
use etheram_node::builders::etheram_node_builder::EtheramNodeBuilder;
use etheram_node::builders::external_interface_incoming_builder::ExternalInterfaceIncomingBuilder;
use etheram_node::builders::external_interface_outgoing_builder::ExternalInterfaceOutgoingBuilder;
use etheram_node::builders::observer_builder::ObserverBuilder;
use etheram_node::builders::partitioner_builder::PartitionerBuilder;
use etheram_node::builders::protocol_builder::ProtocolBuilder;
use etheram_node::builders::storage_builder::StorageBuilder;
use etheram_node::builders::timer_input_builder::TimerInputBuilder;
use etheram_node::builders::timer_output_builder::TimerOutputBuilder;
use etheram_node::builders::transport_incoming_builder::TransportIncomingBuilder;
use etheram_node::builders::transport_outgoing_builder::TransportOutgoingBuilder;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::cache_adapter::CacheAdapter;
use etheram_node::common_types::storage_adapter::StorageAdapter;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::context::context_builder::ContextBuilder;
use etheram_node::etheram_node::EtheramNode;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node::observer::Observer;
use etheram_node::partitioner::partition::Partitioner;

pub struct NodeRuntime {
    node: EtheramNode<()>,
}

impl NodeRuntime {
    pub fn new(peer_id: PeerId) -> Result<Self, String> {
        let node = EtheramNodeBuilder::<()>::new()
            .with_peer_id(peer_id)
            .with_timer_input(build_timer_input()?)
            .with_timer_output(build_timer_output()?)
            .with_transport_incoming(build_transport_incoming()?)
            .with_transport_outgoing(build_transport_outgoing()?)
            .with_external_interface_incoming(build_external_interface_incoming()?)
            .with_external_interface_outgoing(build_external_interface_outgoing()?)
            .with_storage(build_storage()?)
            .with_cache(build_cache()?)
            .with_protocol(build_protocol()?)
            .with_context_builder(build_context_builder()?)
            .with_partitioner(build_partitioner()?)
            .with_observer(build_observer()?)
            .build()
            .map_err(|error| format!("failed to build node runtime: {error:?}"))?;
        Ok(Self { node })
    }

    pub fn run_steps(&mut self, step_limit: u64) -> u64 {
        let mut executed = 0;
        while executed < step_limit {
            if !self.node.step() {
                break;
            }
            executed += 1;
        }
        executed
    }
}

fn build_cache() -> Result<Box<dyn CacheAdapter<Key = (), Value = Transaction>>, String> {
    CacheBuilder::default()
        .build()
        .map_err(|error| format!("failed to build cache: {error:?}"))
}

fn build_context_builder() -> Result<Box<dyn ContextBuilder<()>>, String> {
    ContextBuilderBuilder::default()
        .build()
        .map_err(|error| format!("failed to build context builder: {error:?}"))
}

fn build_external_interface_incoming(
) -> Result<Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>, String> {
    ExternalInterfaceIncomingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build external interface incoming: {error:?}"))
}

fn build_external_interface_outgoing(
) -> Result<Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>, String> {
    ExternalInterfaceOutgoingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build external interface outgoing: {error:?}"))
}

fn build_observer() -> Result<Box<dyn Observer>, String> {
    ObserverBuilder::default()
        .build()
        .map_err(|error| format!("failed to build observer: {error:?}"))
}

fn build_partitioner() -> Result<Box<dyn Partitioner<()>>, String> {
    PartitionerBuilder::default()
        .build()
        .map_err(|error| format!("failed to build partitioner: {error:?}"))
}

fn build_protocol() -> Result<BoxedProtocol<()>, String> {
    ProtocolBuilder::default()
        .build()
        .map_err(|error| format!("failed to build protocol: {error:?}"))
}

fn build_storage() -> Result<Box<dyn StorageAdapter<Key = Address, Value = Account>>, String> {
    StorageBuilder::default()
        .build()
        .map_err(|error| format!("failed to build storage: {error:?}"))
}

fn build_timer_input() -> Result<Box<dyn TimerInputAdapter<TimerEvent>>, String> {
    TimerInputBuilder::default()
        .build()
        .map_err(|error| format!("failed to build timer input: {error:?}"))
}

fn build_timer_output() -> Result<Box<dyn TimerOutputAdapter<TimerEvent, u64>>, String> {
    TimerOutputBuilder::default()
        .build()
        .map_err(|error| format!("failed to build timer output: {error:?}"))
}

fn build_transport_incoming() -> Result<Box<dyn TransportIncomingAdapter<()>>, String> {
    TransportIncomingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build transport incoming: {error:?}"))
}

fn build_transport_outgoing() -> Result<Box<dyn TransportOutgoingAdapter<()>>, String> {
    TransportOutgoingBuilder::default()
        .build()
        .map_err(|error| format!("failed to build transport outgoing: {error:?}"))
}
