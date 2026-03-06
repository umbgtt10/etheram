// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node_process::infra::transport::partitionable_transport::partition_control::apply_control_line;
use etheram_node_process::infra::transport::partitionable_transport::partition_table::PartitionTable;
use etheram_node_process::infra::transport::partitionable_transport::shutdown_signal::is_shutdown_requested;
use etheram_node_process::infra::transport::partitionable_transport::shutdown_signal::reset_shutdown;

#[test]
fn apply_control_line_partition_blocks_link() {
    // Arrange
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "partition 1 2");

    // Assert
    assert!(result.is_ok());
    assert!(table.is_blocked(1, 2));
}

#[test]
fn apply_control_line_heal_unblocks_link() {
    // Arrange
    let table = PartitionTable::new();
    table.block(1, 2);

    // Act
    let result = apply_control_line(&table, "heal 1 2");

    // Assert
    assert!(result.is_ok());
    assert!(!table.is_blocked(1, 2));
}

#[test]
fn apply_control_line_clear_removes_all_partitions() {
    // Arrange
    let table = PartitionTable::new();
    table.block(1, 2);
    table.block(3, 4);

    // Act
    let result = apply_control_line(&table, "clear");

    // Assert
    assert!(result.is_ok());
    assert!(!table.is_blocked(1, 2));
    assert!(!table.is_blocked(3, 4));
}

#[test]
fn apply_control_line_shutdown_sets_shutdown_signal() {
    // Arrange
    reset_shutdown();
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "shutdown");

    // Assert
    assert!(result.is_ok());
    assert!(is_shutdown_requested());
    reset_shutdown();
}

#[test]
fn apply_control_line_unknown_command_returns_error() {
    // Arrange
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "explode 1 2");

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown command"));
}

#[test]
fn apply_control_line_partition_missing_from_returns_error() {
    // Arrange
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "partition");

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing from peer id"));
}

#[test]
fn apply_control_line_partition_missing_to_returns_error() {
    // Arrange
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "partition 1");

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing to peer id"));
}

#[test]
fn apply_control_line_partition_invalid_peer_id_returns_error() {
    // Arrange
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "partition abc 2");

    // Assert
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid from peer id"));
}

#[test]
fn apply_control_line_empty_line_does_nothing() {
    // Arrange
    let table = PartitionTable::new();

    // Act
    let result = apply_control_line(&table, "");

    // Assert
    assert!(result.is_ok());
}
