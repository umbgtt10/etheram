// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod cluster_config;
mod etheram_node;
mod infra;

use crate::cluster_config::ClusterConfig;
use crate::etheram_node::NodeRuntime;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::process::ExitCode;

fn parse_u64(value: &str, field_name: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|error| format!("invalid {field_name} '{}': {error}", value))
}

fn run() -> Result<(), String> {
    let mut args = env::args();
    let _binary_name = args.next();
    let Some(config_path) = args.next() else {
        return Err(
            "usage: etheram-node-process <cluster.toml> <node-id> [step-limit]".to_string(),
        );
    };
    let Some(node_id_arg) = args.next() else {
        return Err(
            "usage: etheram-node-process <cluster.toml> <node-id> [step-limit]".to_string(),
        );
    };
    let step_limit_arg = args.next();

    let node_id = parse_u64(&node_id_arg, "node-id")?;
    let step_limit = match step_limit_arg {
        Some(raw) => parse_u64(&raw, "step-limit")?,
        None => 0,
    };

    let config = ClusterConfig::load_from_path(Path::new(&config_path))?;
    config.validate()?;
    let node = config.find_node(node_id)?;
    let mut peer_addresses = BTreeMap::new();
    for configured_node in &config.node {
        peer_addresses.insert(configured_node.id, configured_node.transport_addr.clone());
    }

    println!(
        "etheram-node-process bootstrap: node_id={}, transport_addr={}, client_addr={}, db_path={}, validators={}, log_level={}",
        node.id,
        node.transport_addr,
        node.client_addr,
        node.db_path,
        config.fleet.validator_set.len(),
        config.fleet.log_level
    );

    let mut runtime = NodeRuntime::new(node.id, &node.transport_addr, &peer_addresses)?;
    if step_limit == 0 {
        println!("etheram-node-process loop mode=forever");
        runtime.run_forever();
        return Ok(());
    }
    let executed_steps = runtime.run_steps(step_limit);
    println!(
        "etheram-node-process loop mode=finite executed_steps={}",
        executed_steps
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
