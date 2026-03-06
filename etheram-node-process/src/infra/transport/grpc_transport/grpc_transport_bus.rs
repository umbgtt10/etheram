// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::transport_service_server::TransportService;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::transport_service_server::TransportServiceServer;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::TransportAck;
use crate::infra::transport::grpc_transport::grpc_transport_proto::wire::TransportEnvelope;
use etheram_core::types::PeerId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::vec::Vec;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

type TransportQueue = VecDeque<(PeerId, Vec<u8>)>;
type InboundState = BTreeMap<PeerId, TransportQueue>;

pub struct GrpcTransportBus {
    inbound: Mutex<InboundState>,
    started_servers: Mutex<BTreeSet<PeerId>>,
}

impl GrpcTransportBus {
    pub fn new() -> Self {
        Self {
            inbound: Mutex::new(BTreeMap::new()),
            started_servers: Mutex::new(BTreeSet::new()),
        }
    }

    pub fn dequeue_for(&self, node_id: PeerId) -> Option<(PeerId, Vec<u8>)> {
        let mut guard = self
            .inbound
            .lock()
            .expect("transport inbound lock poisoned");
        let queue = guard.entry(node_id).or_default();
        queue.pop_front()
    }

    pub fn enqueue_to_local(&self, to_peer: PeerId, from_peer: PeerId, payload: Vec<u8>) {
        let mut guard = self
            .inbound
            .lock()
            .expect("transport inbound lock poisoned");
        let queue = guard.entry(to_peer).or_default();
        queue.push_back((from_peer, payload));
    }

    pub fn ensure_server_started(
        self: &Arc<Self>,
        node_id: PeerId,
        listen_addr: &str,
    ) -> Result<(), String> {
        {
            let mut started = self
                .started_servers
                .lock()
                .map_err(|_| "server state lock poisoned".to_string())?;
            if started.contains(&node_id) {
                return Ok(());
            }
            started.insert(node_id);
        }

        let bus = Arc::clone(self);
        let listen_addr_owned = listen_addr.to_string();
        thread::Builder::new()
            .name(format!("grpc-transport-server-{}", node_id))
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(value) => value,
                    Err(error) => {
                        println!("grpc_server_error node_id={} error={}", node_id, error);
                        return;
                    }
                };

                runtime.block_on(async move {
                    let socket_addr = match listen_addr_owned.parse() {
                        Ok(value) => value,
                        Err(error) => {
                            println!(
                                "grpc_server_error node_id={} invalid_listen_addr={} error={}",
                                node_id, listen_addr_owned, error
                            );
                            return;
                        }
                    };

                    let service = GrpcTransportService { node_id, bus };
                    if let Err(error) = Server::builder()
                        .add_service(TransportServiceServer::new(service))
                        .serve(socket_addr)
                        .await
                    {
                        println!("grpc_server_error node_id={} error={}", node_id, error);
                    }
                });
            })
            .map(|_| ())
            .map_err(|error| format!("failed to spawn grpc server thread: {error}"))
    }
}

impl Default for GrpcTransportBus {
    fn default() -> Self {
        Self::new()
    }
}

struct GrpcTransportService {
    node_id: PeerId,
    bus: Arc<GrpcTransportBus>,
}

#[tonic::async_trait]
impl TransportService for GrpcTransportService {
    async fn send_envelope(
        &self,
        request: Request<TransportEnvelope>,
    ) -> Result<Response<TransportAck>, Status> {
        let envelope = request.into_inner();
        self.bus
            .enqueue_to_local(self.node_id, envelope.from_peer_id, envelope.ibft_message);
        Ok(Response::new(TransportAck { accepted: true }))
    }
}
