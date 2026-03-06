// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn main_missing_arguments_returns_non_success_exit_code() {
    // Arrange
    let mut command = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"));

    // Act
    let output = command
        .output()
        .expect("failed to run etheram-node-process");

    // Assert
    assert!(!output.status.success());
}

#[test]
fn main_valid_arguments_returns_success_exit_code() {
    // Arrange
    let config_path = create_test_config();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"));
    command.arg(&config_path);
    command.arg("1");
    command.arg("1");

    // Act
    let output = command
        .output()
        .expect("failed to run etheram-node-process");

    // Assert
    assert!(output.status.success());

    let _ = fs::remove_file(config_path);
}

#[test]
fn main_valid_arguments_with_grpc_backend_returns_success_exit_code() {
    // Arrange
    let config_path = create_test_config();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"));
    command.arg(&config_path);
    command.arg("1");
    command.arg("1");
    command.env("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND", "grpc");

    // Act
    let output = command
        .output()
        .expect("failed to run etheram-node-process");

    // Assert
    assert!(output.status.success());

    let _ = fs::remove_file(config_path);
}

#[test]
fn main_invalid_partition_env_with_grpc_backend_returns_non_success_exit_code() {
    // Arrange
    let config_path = create_test_config();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"));
    command.arg(&config_path);
    command.arg("1");
    command.arg("1");
    command.env("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND", "grpc");
    command.env("ETHERAM_PARTITION_BLOCKS", "invalid-pair-format");

    // Act
    let output = command
        .output()
        .expect("failed to run etheram-node-process");

    // Assert
    assert!(!output.status.success());

    let _ = fs::remove_file(config_path);
}

fn create_test_config() -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();
    path.push(format!("etheram_node_process_test_{}.toml", millis));

    let config = r#"[fleet]
validator_set = [1, 2, 3, 4, 5]
log_level = "info"

[[node]]
id = 1
transport_addr = "127.0.0.1:7001"
client_addr = "127.0.0.1:8001"
db_path = "./data/node1"
"#;

    fs::write(&path, config).expect("failed to write temporary config file");
    path
}
