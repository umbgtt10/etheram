// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_config::ClusterConfig;
use crate::cluster_config::NodeConfig;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

const NODE_PROCESS_BIN_ENV: &str = "ETHERAM_NODE_PROCESS_BIN";
const NODE_STEP_LIMIT_ENV: &str = "ETHERAM_DESKTOP_NODE_STEP_LIMIT";
const SHUTDOWN_WAIT_ATTEMPTS: usize = 25;
const SHUTDOWN_WAIT_INTERVAL_MS: u64 = 20;

enum ControlCommand {
    Partition(u64, u64),
    Heal(u64, u64),
    Clear,
    Shutdown,
}

pub struct Launcher;

pub struct NodeLogLine {
    pub node_id: u64,
    pub line: String,
}

pub struct LaunchedNode {
    pub node_id: u64,
    child: Child,
    stdin: ChildStdin,
    stdout: Option<BufReader<ChildStdout>>,
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
        println!(
            "desktop_control commands: partition <from> <to> | heal <from> <to> | clear | shutdown"
        );
        Self::run_command_loop(&mut launched)?;
        Self::stop_all(launched)?;
        Ok(())
    }

    pub fn run_command_loop(nodes: &mut [LaunchedNode]) -> Result<(), String> {
        for line_result in stdin().lock().lines() {
            let line =
                line_result.map_err(|error| format!("failed reading desktop command: {error}"))?;
            let command = match Self::parse_control_command(&line)? {
                Some(value) => value,
                None => continue,
            };
            match command {
                ControlCommand::Partition(from, to) => {
                    Self::broadcast_partition_command(nodes, from, to)?
                }
                ControlCommand::Heal(from, to) => Self::broadcast_heal_command(nodes, from, to)?,
                ControlCommand::Clear => Self::broadcast_clear_command(nodes)?,
                ControlCommand::Shutdown => {
                    Self::broadcast_shutdown_command(nodes)?;
                    break;
                }
            }
        }
        Ok(())
    }

    fn parse_control_command(line: &str) -> Result<Option<ControlCommand>, String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        let mut parts = trimmed.split_whitespace();
        let Some(command) = parts.next() else {
            return Ok(None);
        };

        match command {
            "partition" => {
                let from = parse_peer(parts.next(), "from")?;
                let to = parse_peer(parts.next(), "to")?;
                Ok(Some(ControlCommand::Partition(from, to)))
            }
            "heal" => {
                let from = parse_peer(parts.next(), "from")?;
                let to = parse_peer(parts.next(), "to")?;
                Ok(Some(ControlCommand::Heal(from, to)))
            }
            "clear" => Ok(Some(ControlCommand::Clear)),
            "shutdown" => Ok(Some(ControlCommand::Shutdown)),
            other => Err(format!(
                "unknown command '{}', expected partition|heal|clear|shutdown",
                other
            )),
        }
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
            stdout: Some(BufReader::new(stdout)),
        })
    }

    pub fn start_log_pump(nodes: &mut [LaunchedNode]) -> Receiver<NodeLogLine> {
        let (sender, receiver) = channel::<NodeLogLine>();
        for node in nodes {
            let Some(mut stdout) = node.stdout.take() else {
                continue;
            };
            let node_id = node.node_id;
            let sender_for_node = sender.clone();
            let _ = thread::Builder::new()
                .name(format!("desktop-log-pump-{}", node_id))
                .spawn(move || loop {
                    let mut line = String::new();
                    match stdout.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
                            if trimmed.is_empty() {
                                continue;
                            }
                            if sender_for_node
                                .send(NodeLogLine {
                                    node_id,
                                    line: trimmed,
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                });
        }
        receiver
    }

    pub fn send_partition_command(
        node: &mut LaunchedNode,
        from: u64,
        to: u64,
    ) -> Result<(), String> {
        let command = format!("partition {} {}\n", from, to);
        Self::send_raw_command(node, &command)
    }

    pub fn send_heal_command(node: &mut LaunchedNode, from: u64, to: u64) -> Result<(), String> {
        let command = format!("heal {} {}\n", from, to);
        Self::send_raw_command(node, &command)
    }

    pub fn send_clear_command(node: &mut LaunchedNode) -> Result<(), String> {
        Self::send_raw_command(node, "clear\n")
    }

    pub fn send_raw_command(node: &mut LaunchedNode, command: &str) -> Result<(), String> {
        node.stdin.write_all(command.as_bytes()).map_err(|error| {
            format!("failed to write command to node {}: {error}", node.node_id)
        })?;
        node.stdin
            .flush()
            .map_err(|error| format!("failed to flush command to node {}: {error}", node.node_id))
    }

    pub fn send_shutdown_command(node: &mut LaunchedNode) -> Result<(), String> {
        Self::send_raw_command(node, "shutdown\n")
    }

    pub fn broadcast_partition_command(
        nodes: &mut [LaunchedNode],
        from: u64,
        to: u64,
    ) -> Result<(), String> {
        for node in nodes {
            Self::send_partition_command(node, from, to)?;
        }
        Ok(())
    }

    pub fn broadcast_heal_command(
        nodes: &mut [LaunchedNode],
        from: u64,
        to: u64,
    ) -> Result<(), String> {
        for node in nodes {
            Self::send_heal_command(node, from, to)?;
        }
        Ok(())
    }

    pub fn broadcast_isolate_node_command(
        nodes: &mut [LaunchedNode],
        target: u64,
    ) -> Result<usize, String> {
        let node_ids: Vec<u64> = nodes.iter().map(|node| node.node_id).collect();
        if !node_ids.contains(&target) {
            return Err(format!("node {} is not running", target));
        }

        let mut link_count = 0usize;
        for peer_id in node_ids {
            if peer_id == target {
                continue;
            }
            Self::broadcast_partition_command(nodes, target, peer_id)?;
            Self::broadcast_partition_command(nodes, peer_id, target)?;
            link_count += 2;
        }
        Ok(link_count)
    }

    pub fn broadcast_heal_isolated_node_command(
        nodes: &mut [LaunchedNode],
        target: u64,
    ) -> Result<usize, String> {
        let node_ids: Vec<u64> = nodes.iter().map(|node| node.node_id).collect();
        if !node_ids.contains(&target) {
            return Err(format!("node {} is not running", target));
        }

        let mut link_count = 0usize;
        for peer_id in node_ids {
            if peer_id == target {
                continue;
            }
            Self::broadcast_heal_command(nodes, target, peer_id)?;
            Self::broadcast_heal_command(nodes, peer_id, target)?;
            link_count += 2;
        }
        Ok(link_count)
    }

    pub fn broadcast_clear_command(nodes: &mut [LaunchedNode]) -> Result<(), String> {
        for node in nodes {
            Self::send_clear_command(node)?;
        }
        Ok(())
    }

    pub fn broadcast_shutdown_command(nodes: &mut [LaunchedNode]) -> Result<(), String> {
        for node in nodes {
            Self::send_shutdown_command(node)?;
        }
        Ok(())
    }

    pub fn read_stdout_line(node: &mut LaunchedNode) -> Result<Option<String>, String> {
        let Some(stdout) = node.stdout.as_mut() else {
            return Err(format!(
                "stdout stream is not available for node {}",
                node.node_id
            ));
        };
        let mut line = String::new();
        let bytes = stdout
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

fn parse_peer(value: Option<&str>, field: &str) -> Result<u64, String> {
    let Some(raw) = value else {
        return Err(format!("missing {} peer id", field));
    };
    raw.parse::<u64>()
        .map_err(|error| format!("invalid {} peer id '{}': {}", field, raw, error))
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
