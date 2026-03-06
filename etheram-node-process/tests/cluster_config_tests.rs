// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node_process::cluster_config::ClusterConfig;
use etheram_node_process::cluster_config::FleetConfig;
use etheram_node_process::cluster_config::NodeConfig;

fn valid_config() -> ClusterConfig {
    ClusterConfig {
        fleet: FleetConfig {
            validator_set: vec![1, 2, 3],
            log_level: "info".to_string(),
        },
        node: vec![NodeConfig {
            id: 1,
            transport_addr: "127.0.0.1:30001".to_string(),
            client_addr: "127.0.0.1:8001".to_string(),
            db_path: "/tmp/node1".to_string(),
        }],
    }
}

#[test]
fn validate_valid_config_returns_ok() {
    // Arrange
    let config = valid_config();

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn validate_empty_validator_set_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.fleet.validator_set = vec![];

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(result.unwrap_err(), "fleet.validator_set must not be empty");
}

#[test]
fn validate_empty_log_level_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.fleet.log_level = "   ".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(result.unwrap_err(), "fleet.log_level must not be empty");
}

#[test]
fn validate_no_nodes_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.node = vec![];

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(
        result.unwrap_err(),
        "at least one [[node]] entry is required"
    );
}

#[test]
fn validate_duplicate_node_id_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.node.push(NodeConfig {
        id: 1,
        transport_addr: "127.0.0.1:30002".to_string(),
        client_addr: "127.0.0.1:8002".to_string(),
        db_path: "/tmp/node2".to_string(),
    });

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(result.unwrap_err(), "duplicate node.id detected: 1");
}

#[test]
fn validate_empty_transport_addr_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.node[0].transport_addr = "  ".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(
        result.unwrap_err(),
        "node 1 transport_addr must not be empty"
    );
}

#[test]
fn validate_empty_client_addr_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.node[0].client_addr = "  ".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(result.unwrap_err(), "node 1 client_addr must not be empty");
}

#[test]
fn validate_empty_db_path_returns_error() {
    // Arrange
    let mut config = valid_config();
    config.node[0].db_path = "  ".to_string();

    // Act
    let result = config.validate();

    // Assert
    assert_eq!(result.unwrap_err(), "node 1 db_path must not be empty");
}

#[test]
fn validate_multiple_valid_nodes_returns_ok() {
    // Arrange
    let mut config = valid_config();
    config.node.push(NodeConfig {
        id: 2,
        transport_addr: "127.0.0.1:30002".to_string(),
        client_addr: "127.0.0.1:8002".to_string(),
        db_path: "/tmp/node2".to_string(),
    });

    // Act
    let result = config.validate();

    // Assert
    assert!(result.is_ok());
}

#[test]
fn find_node_existing_id_returns_node() {
    // Arrange
    let config = valid_config();

    // Act
    let result = config.find_node(1);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().transport_addr, "127.0.0.1:30001");
}

#[test]
fn find_node_missing_id_returns_error() {
    // Arrange
    let config = valid_config();

    // Act
    let result = config.find_node(99);

    // Assert
    assert_eq!(result.unwrap_err(), "node id 99 not found in config");
}
