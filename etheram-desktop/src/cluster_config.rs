// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct ClusterConfig {
    pub fleet: FleetConfig,
    pub node: Vec<NodeConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FleetConfig {
    pub validator_set: Vec<u64>,
    pub log_level: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NodeConfig {
    pub id: u64,
    pub transport_addr: String,
    pub client_addr: String,
    pub db_path: String,
}

impl ClusterConfig {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read config {}: {error}", path.display()))?;
        toml::from_str::<Self>(&content)
            .map_err(|error| format!("failed to parse config {}: {error}", path.display()))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.fleet.validator_set.is_empty() {
            return Err("fleet.validator_set must not be empty".to_string());
        }
        if self.fleet.log_level.trim().is_empty() {
            return Err("fleet.log_level must not be empty".to_string());
        }
        if self.node.is_empty() {
            return Err("at least one [[node]] entry is required".to_string());
        }

        let mut ids = BTreeSet::new();
        for node in &self.node {
            if !ids.insert(node.id) {
                return Err(format!("duplicate node.id detected: {}", node.id));
            }
            if node.transport_addr.trim().is_empty() {
                return Err(format!("node {} transport_addr must not be empty", node.id));
            }
            if node.client_addr.trim().is_empty() {
                return Err(format!("node {} client_addr must not be empty", node.id));
            }
            if node.db_path.trim().is_empty() {
                return Err(format!("node {} db_path must not be empty", node.id));
            }
        }

        Ok(())
    }
}
