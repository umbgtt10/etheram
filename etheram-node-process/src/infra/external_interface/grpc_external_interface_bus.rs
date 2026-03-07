// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::external_interface::grpc_external_interface_codec::GrpcExternalInterfaceCodec;
use crate::infra::external_interface::grpc_external_interface_proto::wire::external_interface_service_server::ExternalInterfaceService;
use crate::infra::external_interface::grpc_external_interface_proto::wire::external_interface_service_server::ExternalInterfaceServiceServer;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetBalanceReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetBalanceRequest;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetBlockReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetBlockRequest;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetHeightReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::GetHeightRequest;
use crate::infra::external_interface::grpc_external_interface_proto::wire::SubmitTransactionReply;
use crate::infra::external_interface::grpc_external_interface_proto::wire::SubmitTransactionRequest;
use crate::infra::storage::sled_storage::SledStorage;
use etheram_core::storage::Storage;
use etheram_core::types::ClientId;
use etheram_node::common_types::block::Block;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::state::storage::storage_query::StorageQuery;
use etheram_node::state::storage::storage_query_result::StorageQueryResult;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::oneshot;
use tokio::time::sleep;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

const RESPONSE_POLL_INTERVAL_MS: u64 = 10;
const RESPONSE_TIMEOUT_MS: u64 = 2_000;
const SERVER_READY_TIMEOUT_MS: u64 = 2_000;

type SharedGrpcExternalInterfaceState = Arc<Mutex<GrpcExternalInterfaceState>>;

#[derive(Clone)]
pub struct GrpcExternalInterfaceBus {
    state: SharedGrpcExternalInterfaceState,
    shutdown: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    server_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    storage: SledStorage,
}

struct GrpcExternalInterfaceServiceImpl {
    bus: GrpcExternalInterfaceBus,
}

struct GrpcExternalInterfaceState {
    next_client_id: ClientId,
    requests: VecDeque<(ClientId, ClientRequest)>,
    responses: BTreeMap<ClientId, ClientResponse>,
}

impl GrpcExternalInterfaceBus {
    pub fn new(client_addr: &str, storage: SledStorage) -> Result<Self, String> {
        let bus = Self {
            state: Arc::new(Mutex::new(GrpcExternalInterfaceState::new())),
            shutdown: Arc::new(Mutex::new(None)),
            server_thread: Arc::new(Mutex::new(None)),
            storage,
        };
        bus.start_server(client_addr)?;
        Ok(bus)
    }

    pub fn enqueue_request(&self, request: ClientRequest) -> ClientId {
        let mut guard = self.state.lock().expect("external interface lock poisoned");
        let client_id = guard.next_client_id;
        guard.next_client_id = guard.next_client_id.wrapping_add(1);
        guard.requests.push_back((client_id, request));
        client_id
    }

    pub fn load_block(&self, height: u64) -> Option<Block> {
        match self.storage.query(StorageQuery::GetBlock(height)) {
            StorageQueryResult::Block(block) => block,
            _ => None,
        }
    }

    pub fn poll_request(&self) -> Option<(ClientId, ClientRequest)> {
        let mut guard = self.state.lock().expect("external interface lock poisoned");
        guard.requests.pop_front()
    }

    pub fn store_response(&self, client_id: ClientId, response: ClientResponse) {
        let mut guard = self.state.lock().expect("external interface lock poisoned");
        guard.responses.insert(client_id, response);
    }

    pub fn take_response(&self, client_id: ClientId) -> Option<ClientResponse> {
        let mut guard = self.state.lock().expect("external interface lock poisoned");
        guard.responses.remove(&client_id)
    }

    pub fn shutdown(&self) {
        if let Some(sender) = self
            .shutdown
            .lock()
            .expect("external interface shutdown lock poisoned")
            .take()
        {
            let _ = sender.send(());
        }

        if let Some(server_thread) = self
            .server_thread
            .lock()
            .expect("external interface server thread lock poisoned")
            .take()
        {
            let _ = server_thread.join();
        }
    }

    fn start_server(&self, client_addr: &str) -> Result<(), String> {
        let addr: SocketAddr = client_addr
            .parse()
            .map_err(|error| format!("invalid client interface addr {}: {error}", client_addr))?;
        let addr_text = client_addr.to_string();
        let service = GrpcExternalInterfaceServiceImpl::new(self.clone());
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        *self
            .shutdown
            .lock()
            .expect("external interface shutdown lock poisoned") = Some(shutdown_tx);
        let server_thread = thread::Builder::new()
            .name(format!("grpc-external-interface-{}", addr.port()))
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed building grpc external interface runtime");
                runtime.block_on(async move {
                    if let Err(error) = Server::builder()
                        .add_service(ExternalInterfaceServiceServer::new(service))
                        .serve_with_shutdown(addr, async move {
                            let _ = shutdown_rx.await;
                        })
                        .await
                    {
                        println!(
                            "grpc_external_interface_server_error addr={} error={}",
                            addr_text, error
                        );
                    }
                });
            })
            .map_err(|error| format!("failed to spawn external interface server: {error}"))?;
        *self
            .server_thread
            .lock()
            .expect("external interface server thread lock poisoned") = Some(server_thread);

        let started = Instant::now();
        loop {
            match TcpStream::connect_timeout(&addr, Duration::from_millis(25)) {
                Ok(stream) => {
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    return Ok(());
                }
                Err(_) if started.elapsed() <= Duration::from_millis(SERVER_READY_TIMEOUT_MS) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(format!(
                        "external interface server {} did not become ready: {}",
                        client_addr, error
                    ));
                }
            }
        }
    }
}

impl GrpcExternalInterfaceServiceImpl {
    fn new(bus: GrpcExternalInterfaceBus) -> Self {
        Self { bus }
    }

    async fn await_response(&self, request: ClientRequest) -> Result<ClientResponse, Status> {
        let client_id = self.bus.enqueue_request(request);
        let started = Instant::now();
        loop {
            if let Some(response) = self.bus.take_response(client_id) {
                return Ok(response);
            }
            if started.elapsed() > Duration::from_millis(RESPONSE_TIMEOUT_MS) {
                return Err(Status::deadline_exceeded(
                    "timed out awaiting client response",
                ));
            }
            sleep(Duration::from_millis(RESPONSE_POLL_INTERVAL_MS)).await;
        }
    }
}

impl GrpcExternalInterfaceState {
    fn new() -> Self {
        Self {
            next_client_id: 1,
            requests: VecDeque::new(),
            responses: BTreeMap::new(),
        }
    }
}

#[tonic::async_trait]
impl ExternalInterfaceService for GrpcExternalInterfaceServiceImpl {
    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceReply>, Status> {
        let address = GrpcExternalInterfaceCodec::decode_address(&request.into_inner().address)
            .map_err(Status::invalid_argument)?;
        let response = self
            .await_response(ClientRequest::GetBalance(address))
            .await?;
        let reply = GrpcExternalInterfaceCodec::encode_balance_response(&response)
            .map_err(Status::internal)?;
        Ok(Response::new(reply))
    }

    async fn get_block(
        &self,
        request: Request<GetBlockRequest>,
    ) -> Result<Response<GetBlockReply>, Status> {
        let height = request.into_inner().height;
        let block = self.bus.load_block(height);
        Ok(Response::new(GetBlockReply {
            found: block.is_some(),
            block: block.as_ref().map(GrpcExternalInterfaceCodec::encode_block),
        }))
    }

    async fn get_height(
        &self,
        _request: Request<GetHeightRequest>,
    ) -> Result<Response<GetHeightReply>, Status> {
        let response = self.await_response(ClientRequest::GetHeight).await?;
        let reply = GrpcExternalInterfaceCodec::encode_height_response(&response)
            .map_err(Status::internal)?;
        Ok(Response::new(reply))
    }

    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> Result<Response<SubmitTransactionReply>, Status> {
        let transaction = request
            .into_inner()
            .transaction
            .ok_or_else(|| Status::invalid_argument("missing transaction payload"))
            .and_then(|transaction| {
                GrpcExternalInterfaceCodec::decode_transaction(&transaction)
                    .map_err(Status::invalid_argument)
            })?;
        let response = self
            .await_response(ClientRequest::SubmitTransaction(transaction))
            .await?;
        let reply = GrpcExternalInterfaceCodec::encode_submit_transaction_response(&response)
            .map_err(Status::internal)?;
        Ok(Response::new(reply))
    }
}
