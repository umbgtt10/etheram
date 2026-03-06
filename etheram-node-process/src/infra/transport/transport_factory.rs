// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_bus::GrpcTransportBus;
use crate::infra::transport::grpc_transport::grpc_transport_incoming::GrpcTransportIncoming;
use crate::infra::transport::grpc_transport::grpc_transport_outgoing::GrpcTransportOutgoing;
use crate::infra::transport::grpc_transport::sync_bus::SyncBus;
use crate::infra::transport::partitionable_transport::partition_table::PartitionTable;
use crate::infra::transport::transport_backend::TransportBackend;
use crate::infra::transport::transport_incoming_factory::build_transport_incoming as build_local_transport_incoming;
use crate::infra::transport::transport_outgoing_factory::build_transport_outgoing as build_local_transport_outgoing;
use etheram_core::node_common::transport_incoming_adapter::TransportIncomingAdapter;
use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use std::collections::BTreeMap;
use std::sync::Arc;

pub fn build_transport_incoming(
    backend: &TransportBackend,
    node_id: PeerId,
    listen_addr: &str,
    bus: Arc<GrpcTransportBus>,
    sync_bus: Arc<SyncBus>,
) -> Result<Box<dyn TransportIncomingAdapter<IbftMessage>>, String> {
    match backend {
        TransportBackend::LocalNoOp => build_local_transport_incoming(),
        TransportBackend::Grpc => Ok(Box::new(GrpcTransportIncoming::new(
            node_id,
            listen_addr.to_string(),
            bus,
            sync_bus,
        )?)),
    }
}

pub fn build_transport_outgoing(
    backend: &TransportBackend,
    node_id: PeerId,
    peer_addresses: &BTreeMap<PeerId, String>,
    partition_table: Arc<PartitionTable>,
    bus: Arc<GrpcTransportBus>,
) -> Result<Box<dyn TransportOutgoingAdapter<IbftMessage>>, String> {
    match backend {
        TransportBackend::LocalNoOp => build_local_transport_outgoing(),
        TransportBackend::Grpc => Ok(Box::new(GrpcTransportOutgoing::new(
            node_id,
            peer_addresses.clone(),
            partition_table,
            bus,
        ))),
    }
}
