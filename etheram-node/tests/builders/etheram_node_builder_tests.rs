// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::builders::{
    cache_builder::CacheBuilder, context_builder_builder::ContextBuilderBuilder,
    etheram_node_builder::EtheramNodeBuilder,
    external_interface_incoming_builder::ExternalInterfaceIncomingBuilder,
    external_interface_outgoing_builder::ExternalInterfaceOutgoingBuilder,
    observer_builder::ObserverBuilder, partitioner_builder::PartitionerBuilder,
    protocol_builder::ProtocolBuilder, storage_builder::StorageBuilder,
    timer_input_builder::TimerInputBuilder, timer_output_builder::TimerOutputBuilder,
    transport_incoming_builder::TransportIncomingBuilder,
    transport_outgoing_builder::TransportOutgoingBuilder,
};

#[test]
fn build_missing_peer_id_returns_error() {
    // Arrange
    let builder = EtheramNodeBuilder::<()>::new();

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_err());
}

#[test]
fn build_with_all_components_builds_successfully() {
    // Arrange
    let builder = EtheramNodeBuilder::<()>::new()
        .with_peer_id(1)
        .with_timer_input(TimerInputBuilder::default().build().unwrap())
        .with_timer_output(TimerOutputBuilder::default().build().unwrap())
        .with_transport_incoming(TransportIncomingBuilder::default().build().unwrap())
        .with_transport_outgoing(TransportOutgoingBuilder::default().build().unwrap())
        .with_external_interface_incoming(
            ExternalInterfaceIncomingBuilder::default().build().unwrap(),
        )
        .with_external_interface_outgoing(
            ExternalInterfaceOutgoingBuilder::default().build().unwrap(),
        )
        .with_storage(StorageBuilder::default().build().unwrap())
        .with_cache(CacheBuilder::default().build().unwrap())
        .with_protocol(ProtocolBuilder::default().build().unwrap())
        .with_context_builder(ContextBuilderBuilder::default().build().unwrap())
        .with_partitioner(PartitionerBuilder::default().build().unwrap())
        .with_observer(ObserverBuilder::default().build().unwrap());

    // Act
    let result = builder.build();

    // Assert
    assert!(result.is_ok());
}
