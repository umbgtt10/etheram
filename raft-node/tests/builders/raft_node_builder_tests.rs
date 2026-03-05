// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::timer_input::TimerInput;
use etheram_core::timer_output::TimerOutput;
use raft_node::builders::error::BuildError;
use raft_node::builders::raft_node_builder::RaftNodeBuilder;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

struct DummyTimer;

impl TimerInput for DummyTimer {
    type Event = RaftTimerEvent;

    fn poll(&self) -> Option<Self::Event> {
        None
    }
}

impl TimerOutput for DummyTimer {
    type Event = RaftTimerEvent;
    type Duration = u64;

    fn schedule(&self, _event: Self::Event, _delay: Self::Duration) {}
}

struct DummyExternalInterface;

impl ExternalInterfaceIncoming for DummyExternalInterface {
    type Request = RaftClientRequest;

    fn poll_request(&self) -> Option<(u64, Self::Request)> {
        None
    }
}

impl ExternalInterfaceOutgoing for DummyExternalInterface {
    type Response = RaftClientResponse;

    fn send_response(&self, _client_id: u64, _response: Self::Response) {}
}

#[test]
fn build_missing_peer_id_returns_missing_component_error() {
    // Arrange
    let builder = RaftNodeBuilder::<Vec<u8>>::new()
        .with_timer_input(Box::new(DummyTimer))
        .with_timer_output(Box::new(DummyTimer))
        .with_ei_incoming(Box::new(DummyExternalInterface))
        .with_ei_outgoing(Box::new(DummyExternalInterface));

    // Act
    let result = builder.build();

    // Assert
    assert!(matches!(
        result,
        Err(BuildError::MissingComponent("peer_id"))
    ));
}

#[test]
fn build_missing_timer_input_returns_missing_component_error() {
    // Arrange
    let builder = RaftNodeBuilder::<Vec<u8>>::new()
        .with_peer_id(1)
        .with_timer_output(Box::new(DummyTimer))
        .with_ei_incoming(Box::new(DummyExternalInterface))
        .with_ei_outgoing(Box::new(DummyExternalInterface));

    // Act
    let result = builder.build();

    // Assert
    assert!(matches!(
        result,
        Err(BuildError::MissingComponent("timer_input"))
    ));
}

#[test]
fn build_with_required_components_returns_ok_node() {
    // Arrange
    let builder = RaftNodeBuilder::<Vec<u8>>::new()
        .with_peer_id(1)
        .with_timer_input(Box::new(DummyTimer))
        .with_timer_output(Box::new(DummyTimer))
        .with_ei_incoming(Box::new(DummyExternalInterface))
        .with_ei_outgoing(Box::new(DummyExternalInterface));

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn build_missing_ei_outgoing_returns_missing_component_error() {
    // Arrange
    let builder = RaftNodeBuilder::<Vec<u8>>::new()
        .with_peer_id(1)
        .with_timer_input(Box::new(DummyTimer))
        .with_timer_output(Box::new(DummyTimer))
        .with_ei_incoming(Box::new(DummyExternalInterface));

    // Act
    let result = builder.build();

    // Assert
    assert!(matches!(
        result,
        Err(BuildError::MissingComponent("ei_outgoing"))
    ));
}
