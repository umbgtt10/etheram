// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;
use etheram::{
    brain::protocol::boxed_protocol::BoxedProtocol,
    common_types::{
        account::Account, cache_adapter::CacheAdapter,
        external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter,
        external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter,
        storage_adapter::StorageAdapter, timer_input_adapter::TimerInputAdapter,
        timer_output_adapter::TimerOutputAdapter, transaction::Transaction,
        transport_incoming_adapter::TransportInputAdapter,
        transport_outgoing_adapter::TransportOutputAdapter, types::Address,
    },
    context::context_builder::ContextBuilder,
    execution::execution_engine::BoxedExecutionEngine,
    executor::outgoing::external_interface::client_response::ClientResponse,
    incoming::{external_interface::client_request::ClientRequest, timer::timer_event::TimerEvent},
    observer::Observer,
    partitioner::partition::Partitioner,
};
use etheram_core::types::PeerId;

pub enum StorageVariant {
    InMemory,
    Custom(Box<dyn StorageAdapter<Key = Address, Value = Account>>),
}

pub enum CacheVariant {
    InMemory,
    Custom(Box<dyn CacheAdapter<Key = (), Value = Transaction>>),
}

pub enum ProtocolVariant<M> {
    NoOp,
    Ibft { validators: Vec<PeerId> },
    Custom(BoxedProtocol<M>),
}

pub enum ContextBuilderVariant {
    Eager,
    Custom(Box<dyn ContextBuilder<()>>),
}

pub enum PartitionerVariant {
    TypeBased,
    Custom(Box<dyn Partitioner<()>>),
}

pub enum ExecutionEngineVariant {
    NoOp,
    TinyEvm,
    ValueTransfer,
    Custom(BoxedExecutionEngine),
}

pub enum TimerInputVariant {
    NoOp,
    Custom(Box<dyn TimerInputAdapter<TimerEvent>>),
}

pub enum TimerOutputVariant {
    NoOp,
    Custom(Box<dyn TimerOutputAdapter<TimerEvent, u64>>),
}

pub enum IncomingTransportVariant {
    NoOp,
    Custom(Box<dyn TransportInputAdapter<()>>),
}

pub enum OutgoingTransportVariant {
    NoOp,
    Custom(Box<dyn TransportOutputAdapter<()>>),
}

pub enum IncomingExternalInterfaceVariant {
    NoOp,
    Custom(Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>),
}

pub enum OutgoingExternalInterfaceVariant {
    NoOp,
    Custom(Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>),
}

pub enum ObserverVariant {
    NoOp,
    Custom(Box<dyn Observer>),
}
