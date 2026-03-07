// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::cleanup_test_config;
use crate::common::test_config::create_test_config;
use std::process::Command;

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
    command.env_remove("ETHERAM_PARTITION_BLOCKS");
    command.env_remove("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND");

    // Act
    let output = command
        .output()
        .expect("failed to run etheram-node-process");

    // Assert
    assert!(output.status.success());

    cleanup_test_config(&config_path);
}

#[test]
fn main_valid_arguments_with_grpc_backend_returns_success_exit_code() {
    // Arrange
    let config_path = create_test_config();
    let mut command = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"));
    command.arg(&config_path);
    command.arg("1");
    command.arg("1");
    command.env_remove("ETHERAM_PARTITION_BLOCKS");
    command.env("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND", "grpc");

    // Act
    let output = command
        .output()
        .expect("failed to run etheram-node-process");

    // Assert
    assert!(output.status.success());

    cleanup_test_config(&config_path);
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

    cleanup_test_config(&config_path);
}
