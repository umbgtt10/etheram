// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_config;
use crate::common::test_config::create_test_config;
use etheram_node_process::cluster_config::ClusterConfig;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::external_interface_service_client::ExternalInterfaceServiceClient;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::GetBlockRequest;
use etheram_node_process::infra::external_interface::grpc_external_interface_proto::wire::GetHeightRequest;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build test runtime")
        .block_on(future)
}

async fn connect_client(addr: &str) -> ExternalInterfaceServiceClient<tonic::transport::Channel> {
    for _ in 0..60 {
        match ExternalInterfaceServiceClient::connect(format!("http://{addr}")).await {
            Ok(client) => return client,
            Err(_) => tokio::time::sleep(Duration::from_millis(20)).await,
        }
    }
    panic!("failed to connect to external interface service")
}

fn read_stderr_to_string(mut stderr: std::process::ChildStderr) -> String {
    let mut buffer = String::new();
    let _ = stderr.read_to_string(&mut buffer);
    buffer
}

#[test]
fn main_run_forever_external_interface_get_height_and_get_block_return_replies() {
    // Arrange
    let config_path = create_test_config();
    let config =
        ClusterConfig::load_from_path(Path::new(&config_path)).expect("failed to load test config");
    let client_addr = config
        .find_node(1)
        .expect("missing node 1")
        .client_addr
        .clone();
    let mut child = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"))
        .arg(&config_path)
        .arg("1")
        .env_remove("ETHERAM_PARTITION_BLOCKS")
        .env_remove("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("failed to spawn node process: {}", error));
    thread::sleep(Duration::from_millis(120));

    // Act
    let height_reply = block_on(async {
        let mut client = connect_client(&client_addr).await;
        client
            .get_height(GetHeightRequest {})
            .await
            .expect("get_height failed")
            .into_inner()
    });
    let block_reply = block_on(async {
        let mut client = connect_client(&client_addr).await;
        client
            .get_block(GetBlockRequest { height: 99 })
            .await
            .expect("get_block failed")
            .into_inner()
    });
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(b"shutdown\n")
            .unwrap_or_else(|error| panic!("failed to send shutdown command: {}", error));
        stdin
            .flush()
            .unwrap_or_else(|error| panic!("failed to flush shutdown command: {}", error));
    } else {
        panic!("child stdin not available");
    }

    // Assert
    assert_eq!(height_reply.height, 0);
    assert!(!block_reply.found);
    assert!(block_reply.block.is_none());
    let started = Instant::now();
    let status = loop {
        if let Some(exit_status) = child
            .try_wait()
            .unwrap_or_else(|error| panic!("failed to poll child status: {}", error))
        {
            break exit_status;
        }
        if started.elapsed() > Duration::from_secs(2) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run-forever process did not exit within timeout after shutdown");
        }
        thread::sleep(Duration::from_millis(20));
    };
    let stderr_output = child
        .stderr
        .take()
        .map(read_stderr_to_string)
        .unwrap_or_default();
    assert!(
        status.success(),
        "expected successful exit, got {} stderr: {}",
        status,
        stderr_output
    );
    cleanup_test_config(&config_path);
}
