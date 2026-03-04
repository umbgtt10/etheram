// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use raft_node::{
    brain::protocol::{boxed_protocol::BoxedRaftProtocol, message::RaftMessage},
    common_types::{
        cache_adapter::CacheAdapter,
        external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter,
        external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter,
        state_machine::RaftStateMachine, storage_adapter::StorageAdapter,
        timer_input_adapter::TimerInputAdapter, timer_output_adapter::TimerOutputAdapter,
        transport_incoming_adapter::TransportIncomingAdapter,
        transport_outgoing_adapter::TransportOutgoingAdapter,
    },
    context::context_builder::RaftContextBuilder,
    executor::outgoing::external_interface::client_response::RaftClientResponse,
    incoming::{
        external_interface::client_request::RaftClientRequest, timer::timer_event::RaftTimerEvent,
    },
    observer::RaftObserver,
    partitioner::partition::RaftPartitioner,
};

pub enum RaftStorageVariant<P: Clone + 'static> {
    InMemory,
    Custom(Box<dyn StorageAdapter<P, Key = (), Value = ()>>),
}

pub enum RaftCacheVariant {
    InMemory,
    Custom(Box<dyn CacheAdapter<Key = (), Value = ()>>),
}

pub enum RaftProtocolVariant<P: Clone + 'static> {
    Raft {
        validator_set: Vec<PeerId>,
        election_timeout_ms: u64,
        heartbeat_interval_ms: u64,
    },
    Custom(BoxedRaftProtocol<P>),
}

pub enum RaftContextBuilderVariant<P: Clone + 'static> {
    Eager,
    Custom(Box<dyn RaftContextBuilder<P>>),
}

pub enum RaftPartitionerVariant<P: Clone + 'static> {
    TypeBased,
    Custom(Box<dyn RaftPartitioner<P>>),
}

pub enum RaftStateMachineVariant {
    InMemory,
    Custom(Box<dyn RaftStateMachine>),
}

pub enum RaftTimerInputVariant {
    Manual,
    Custom(Box<dyn TimerInputAdapter<RaftTimerEvent>>),
}

pub enum RaftTimerOutputVariant {
    Manual,
    Custom(Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>),
}

pub enum RaftTransportIncomingVariant<P: Clone + 'static> {
    InMemory,
    NoOp,
    Custom(Box<dyn TransportIncomingAdapter<RaftMessage<P>>>),
}

pub enum RaftTransportOutgoingVariant<P: Clone + 'static> {
    InMemory,
    NoOp,
    Custom(Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>),
}

pub enum RaftExternalInterfaceIncomingVariant {
    InMemory,
    Custom(Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>),
}

pub enum RaftExternalInterfaceOutgoingVariant {
    InMemory,
    Custom(Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>),
}

pub enum RaftObserverVariant {
    NoOp,
    Custom(Box<dyn RaftObserver>),
}
