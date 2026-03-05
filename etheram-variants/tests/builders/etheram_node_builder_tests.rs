// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_variants::builders::cache_builder::CacheBuilder;
use etheram_variants::builders::context_builder_builder::ContextBuilderBuilder;
use etheram_variants::builders::etheram_node_builder::EtheramNodeBuilder;
use etheram_variants::builders::external_interface_incoming_builder::ExternalInterfaceIncomingBuilder;
use etheram_variants::builders::external_interface_outgoing_builder::ExternalInterfaceOutgoingBuilder;
use etheram_variants::builders::observer_builder::ObserverBuilder;
use etheram_variants::builders::partitioner_builder::PartitionerBuilder;
use etheram_variants::builders::protocol_builder::ProtocolBuilder;
use etheram_variants::builders::storage_builder::StorageBuilder;
use etheram_variants::builders::timer_input_builder::TimerInputBuilder;
use etheram_variants::builders::timer_output_builder::TimerOutputBuilder;
use etheram_variants::builders::transport_incoming_builder::TransportIncomingBuilder;
use etheram_variants::builders::transport_outgoing_builder::TransportOutgoingBuilder;

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
