// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_db_path;
use crate::common::test_config::create_test_db_path;
use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::storage::Storage;
use etheram_node::common_types::block::Block;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::executor::outgoing::external_interface::transaction_rejection_reason::TransactionRejectionReason;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::state::storage::storage_mutation::StorageMutation;
use etheram_node_process::infra::external_interface::grpc_external_interface_bus::GrpcExternalInterfaceBus;
use etheram_node_process::infra::external_interface::grpc_external_interface_codec::GrpcExternalInterfaceCodec;
use etheram_node_process::infra::external_interface::grpc_external_interface_incoming::GrpcExternalInterfaceIncoming;
use etheram_node_process::infra::external_interface::grpc_external_interface_outgoing::GrpcExternalInterfaceOutgoing;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::external_interface_service_client::ExternalInterfaceServiceClient;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::GetBalanceRequest;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::GetBlockRequest;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::GetHeightRequest;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::SubmitTransactionRequest;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::TransactionRejectionReason as WireTransactionRejectionReason;
use etheram_node_process::infra::storage::sled_storage::SledStorage;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const POLL_TIMEOUT_MS: u64 = 2_000;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build test runtime")
        .block_on(future)
}

async fn connect_client(port: u16) -> ExternalInterfaceServiceClient<tonic::transport::Channel> {
    for _ in 0..40 {
        match ExternalInterfaceServiceClient::connect(format!("http://127.0.0.1:{port}")).await {
            Ok(client) => return client,
            Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
        }
    }
    panic!("failed to connect external interface client")
}

fn next_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to allocate local port");
    listener
        .local_addr()
        .expect("failed to get local socket address")
        .port()
}

fn wait_for_polled_request(incoming: &GrpcExternalInterfaceIncoming) -> (u64, ClientRequest) {
    let started = Instant::now();
    loop {
        if let Some(polled) = incoming.poll_request() {
            return polled;
        }
        if started.elapsed() > Duration::from_millis(POLL_TIMEOUT_MS) {
            panic!("timed out waiting for external interface request");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn get_height_rpc_polled_request_then_response_returns_height_reply() {
    // Arrange
    let db_path = create_test_db_path("grpc_external_interface_height");
    let storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let port = next_port();
    let bus = GrpcExternalInterfaceBus::new(&format!("127.0.0.1:{port}"), storage)
        .expect("failed to build external interface bus");
    let incoming = GrpcExternalInterfaceIncoming::new(bus.clone());
    let outgoing = GrpcExternalInterfaceOutgoing::new(bus);

    // Act
    let handle = thread::spawn(move || {
        block_on(async move {
            let mut client = connect_client(port).await;
            client
                .get_height(GetHeightRequest {})
                .await
                .expect("get_height failed")
                .into_inner()
        })
    });
    let (client_id, request) = wait_for_polled_request(&incoming);
    outgoing.send_response(client_id, ClientResponse::Height(7));
    let reply = handle.join().expect("height request thread panicked");

    // Assert
    assert_eq!(request, ClientRequest::GetHeight);
    assert_eq!(reply.height, 7);
    cleanup_test_db_path(&db_path);
}

#[test]
fn get_balance_rpc_polled_request_then_response_returns_balance_reply() {
    // Arrange
    let db_path = create_test_db_path("grpc_external_interface_balance");
    let storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let port = next_port();
    let bus = GrpcExternalInterfaceBus::new(&format!("127.0.0.1:{port}"), storage)
        .expect("failed to build external interface bus");
    let incoming = GrpcExternalInterfaceIncoming::new(bus.clone());
    let outgoing = GrpcExternalInterfaceOutgoing::new(bus);
    let address = [9u8; 20];

    // Act
    let handle = thread::spawn(move || {
        block_on(async move {
            let mut client = connect_client(port).await;
            client
                .get_balance(GetBalanceRequest {
                    address: address.to_vec(),
                })
                .await
                .expect("get_balance failed")
                .into_inner()
        })
    });
    let (client_id, request) = wait_for_polled_request(&incoming);
    outgoing.send_response(
        client_id,
        ClientResponse::Balance {
            balance: 123,
            height: 4,
        },
    );
    let reply = handle.join().expect("balance request thread panicked");

    // Assert
    assert_eq!(request, ClientRequest::GetBalance(address));
    assert_eq!(reply.balance, "123");
    assert_eq!(reply.height, 4);
    cleanup_test_db_path(&db_path);
}

#[test]
fn submit_transaction_rpc_polled_request_then_rejection_returns_rejection_reason() {
    // Arrange
    let db_path = create_test_db_path("grpc_external_interface_submit");
    let storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let port = next_port();
    let bus = GrpcExternalInterfaceBus::new(&format!("127.0.0.1:{port}"), storage)
        .expect("failed to build external interface bus");
    let incoming = GrpcExternalInterfaceIncoming::new(bus.clone());
    let outgoing = GrpcExternalInterfaceOutgoing::new(bus);
    let transaction = Transaction::new([1u8; 20], [2u8; 20], 11, 21_000, 1, 0, vec![1, 2, 3]);
    let wire_transaction = GrpcExternalInterfaceCodec::encode_transaction(&transaction);

    // Act
    let handle = thread::spawn(move || {
        block_on(async move {
            let mut client = connect_client(port).await;
            client
                .submit_transaction(SubmitTransactionRequest {
                    transaction: Some(wire_transaction),
                })
                .await
                .expect("submit_transaction failed")
                .into_inner()
        })
    });
    let (client_id, request) = wait_for_polled_request(&incoming);
    outgoing.send_response(
        client_id,
        ClientResponse::TransactionRejected {
            reason: TransactionRejectionReason::InvalidNonce,
        },
    );
    let reply = handle.join().expect("submit request thread panicked");

    // Assert
    assert_eq!(request, ClientRequest::SubmitTransaction(transaction));
    assert!(!reply.accepted);
    assert_eq!(
        reply.rejection_reason,
        WireTransactionRejectionReason::InvalidNonce as i32
    );
    cleanup_test_db_path(&db_path);
}

#[test]
fn get_block_rpc_existing_height_returns_block_reply() {
    // Arrange
    let db_path = create_test_db_path("grpc_external_interface_block");
    let mut storage =
        SledStorage::new(db_path.to_string_lossy().as_ref()).expect("failed to build storage");
    let block = Block::new(
        3,
        2,
        vec![Transaction::new(
            [1u8; 20],
            [2u8; 20],
            99,
            45_000,
            5,
            1,
            vec![8, 9],
        )],
        [4u8; 32],
        50_000,
    );
    storage.mutate(StorageMutation::StoreBlock(block.clone()));
    let port = next_port();
    let _bus = GrpcExternalInterfaceBus::new(&format!("127.0.0.1:{port}"), storage)
        .expect("failed to build external interface bus");

    // Act
    let reply = block_on(async move {
        let mut client = connect_client(port).await;
        client
            .get_block(GetBlockRequest { height: 3 })
            .await
            .expect("get_block failed")
            .into_inner()
    });

    // Assert
    assert!(reply.found);
    assert!(reply.block.is_some());
    let wire_block = reply.block.expect("expected block payload");
    assert_eq!(wire_block.height, block.height);
    assert_eq!(wire_block.proposer, block.proposer);
    assert_eq!(wire_block.gas_limit, block.gas_limit);
    assert_eq!(wire_block.transactions.len(), 1);
    cleanup_test_db_path(&db_path);
}
