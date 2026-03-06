// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_config::ClusterConfig;
use crate::cluster_config::NodeConfig;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

const NODE_PROCESS_BIN_ENV: &str = "ETHERAM_NODE_PROCESS_BIN";
const NODE_STEP_LIMIT_ENV: &str = "ETHERAM_DESKTOP_NODE_STEP_LIMIT";
const SHUTDOWN_WAIT_ATTEMPTS: usize = 25;
const SHUTDOWN_WAIT_INTERVAL_MS: u64 = 20;

pub struct Launcher;

pub struct LaunchedNode {
    pub node_id: u64,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Launcher {
    pub fn run(config: &ClusterConfig, config_path: &str) -> Result<(), String> {
        config.validate()?;
        println!(
            "etheram-desktop bootstrap: fleet_nodes={}, validators={}",
            config.node.len(),
            config.fleet.validator_set.len()
        );
        let mut launched = Self::spawn_node_processes(config, config_path)?;
        for node in &mut launched {
            while let Some(line) = Self::read_stdout_line(node)? {
                println!("node_stdout id={} line={}", node.node_id, line);
            }
        }
        Self::stop_all(launched)?;
        Ok(())
    }

    pub fn spawn_node_processes(
        config: &ClusterConfig,
        config_path: &str,
    ) -> Result<Vec<LaunchedNode>, String> {
        let step_limit = read_step_limit();
        let mut launched = Vec::new();
        for node in &config.node {
            let args = vec![
                config_path.to_string(),
                node.id.to_string(),
                step_limit.to_string(),
            ];
            let process = Self::spawn_node_with_command(&read_node_process_program(), &args, node)?;
            launched.push(process);
        }
        Ok(launched)
    }

    pub fn spawn_node_with_command(
        program: &str,
        args: &[String],
        node: &NodeConfig,
    ) -> Result<LaunchedNode, String> {
        let mut command = Command::new(program);
        command.args(args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|error| format!("failed to spawn node {} process: {error}", node.id))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| format!("failed to capture stdin for node {}", node.id))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| format!("failed to capture stdout for node {}", node.id))?;
        Ok(LaunchedNode {
            node_id: node.id,
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub fn send_partition_command(
        node: &mut LaunchedNode,
        from: u64,
        to: u64,
    ) -> Result<(), String> {
        let command = format!("partition {} {}\n", from, to);
        node.stdin.write_all(command.as_bytes()).map_err(|error| {
            format!("failed to write command to node {}: {error}", node.node_id)
        })?;
        node.stdin
            .flush()
            .map_err(|error| format!("failed to flush command to node {}: {error}", node.node_id))
    }

    pub fn send_shutdown_command(node: &mut LaunchedNode) -> Result<(), String> {
        node.stdin.write_all(b"shutdown\n").map_err(|error| {
            format!("failed to write shutdown to node {}: {error}", node.node_id)
        })?;
        node.stdin
            .flush()
            .map_err(|error| format!("failed to flush shutdown to node {}: {error}", node.node_id))
    }

    pub fn read_stdout_line(node: &mut LaunchedNode) -> Result<Option<String>, String> {
        let mut line = String::new();
        let bytes = node
            .stdout
            .read_line(&mut line)
            .map_err(|error| format!("failed to read stdout for node {}: {error}", node.node_id))?;
        if bytes == 0 {
            return Ok(None);
        }
        Ok(Some(line.trim_end_matches(['\r', '\n']).to_string()))
    }

    pub fn stop_all(mut nodes: Vec<LaunchedNode>) -> Result<(), String> {
        for node in &mut nodes {
            let _ = Self::send_shutdown_command(node);
            let mut exited = false;
            for _ in 0..SHUTDOWN_WAIT_ATTEMPTS {
                match node.child.try_wait() {
                    Ok(Some(_status)) => {
                        exited = true;
                        break;
                    }
                    Ok(None) => thread::sleep(Duration::from_millis(SHUTDOWN_WAIT_INTERVAL_MS)),
                    Err(error) => {
                        return Err(format!(
                            "failed to poll node {} process state: {}",
                            node.node_id, error
                        ));
                    }
                }
            }
            if !exited {
                let _ = node.child.kill();
            }
            node.child.wait().map_err(|error| {
                format!("failed to wait for node {} process: {error}", node.node_id)
            })?;
        }
        Ok(())
    }
}

fn read_node_process_program() -> String {
    match std::env::var(NODE_PROCESS_BIN_ENV) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "etheram-node-process".to_string(),
    }
}

fn read_step_limit() -> u64 {
    match std::env::var(NODE_STEP_LIMIT_ENV) {
        Ok(value) => value.parse::<u64>().unwrap_or(1),
        Err(_) => 1,
    }
}
