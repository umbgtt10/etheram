// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_desktop::cluster_config::ClusterConfig;
use etheram_desktop::cluster_config::NodeConfig;
use etheram_desktop::launcher::Launcher;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn create_node_config() -> NodeConfig {
    NodeConfig {
        id: 1,
        transport_addr: "127.0.0.1:7001".to_string(),
        client_addr: "127.0.0.1:8001".to_string(),
        db_path: "./data/node1".to_string(),
    }
}

fn spawn_echo_process() -> etheram_desktop::launcher::LaunchedNode {
    let program = "powershell".to_string();
    let args = vec![
        "-NoProfile".to_string(),
        "-Command".to_string(),
        "while (($line = [Console]::In.ReadLine()) -ne $null) { Write-Output \"partition_update $line\"; if ($line -eq \"shutdown\") { break } }"
            .to_string(),
    ];
    let node = create_node_config();
    Launcher::spawn_node_with_command(&program, &args, &node).expect("failed to spawn echo process")
}

fn spawn_echo_processes(count: usize) -> Vec<etheram_desktop::launcher::LaunchedNode> {
    let mut nodes = Vec::new();
    for _ in 0..count {
        nodes.push(spawn_echo_process());
    }
    nodes
}

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = path.pop();
    path
}

fn node_process_binary_path() -> PathBuf {
    if let Ok(value) = env::var("ETHERAM_NODE_PROCESS_BIN") {
        let candidate = PathBuf::from(value);
        if candidate.exists() {
            return candidate;
        }
    }

    let output = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("etheram-node-process")
        .output()
        .expect("failed to run cargo build -p etheram-node-process");
    if !output.status.success() {
        panic!(
            "cargo build -p etheram-node-process failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut path = workspace_root();
    path.push("target");
    path.push("debug");
    path.push("etheram-node-process.exe");
    path
}

fn create_three_node_cluster_config() -> PathBuf {
    let mut path = env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();
    path.push(format!("etheram_desktop_cluster_{}.toml", millis));

    let config = r#"[fleet]
validator_set = [1, 2, 3]
log_level = "info"

[[node]]
id = 1
transport_addr = "127.0.0.1:7801"
client_addr = "127.0.0.1:8801"
db_path = "./data/node1"

[[node]]
id = 2
transport_addr = "127.0.0.1:7802"
client_addr = "127.0.0.1:8802"
db_path = "./data/node2"

[[node]]
id = 3
transport_addr = "127.0.0.1:7803"
client_addr = "127.0.0.1:8803"
db_path = "./data/node3"
"#;

    fs::write(&path, config).expect("failed to write desktop cluster config");
    path
}

fn read_until_contains(
    node: &mut etheram_desktop::launcher::LaunchedNode,
    expected_fragment: &str,
    max_lines: usize,
) -> Result<(), String> {
    for _ in 0..max_lines {
        let Some(line) = Launcher::read_stdout_line(node)? else {
            return Err(format!(
                "node {} stdout closed before fragment '{}' was observed",
                node.node_id, expected_fragment
            ));
        };
        if line.contains(expected_fragment) {
            return Ok(());
        }
    }
    Err(format!(
        "node {} did not emit fragment '{}' within {} lines",
        node.node_id, expected_fragment, max_lines
    ))
}

#[test]
fn send_partition_command_powershell_echo_process_returns_partition_update_line() {
    // Arrange
    let mut launched = spawn_echo_process();

    // Act
    Launcher::send_partition_command(&mut launched, 1, 2)
        .expect("failed to send partition command");
    let line = Launcher::read_stdout_line(&mut launched)
        .expect("failed to read stdout line")
        .expect("expected one stdout line");

    // Assert
    assert_eq!(line, "partition_update partition 1 2");
    Launcher::stop_all(vec![launched]).expect("failed to stop process");
}

#[test]
fn send_shutdown_command_powershell_echo_process_returns_shutdown_line() {
    // Arrange
    let mut launched = spawn_echo_process();

    // Act
    Launcher::send_shutdown_command(&mut launched).expect("failed to send shutdown command");
    let line = Launcher::read_stdout_line(&mut launched)
        .expect("failed to read stdout line")
        .expect("expected one stdout line");

    // Assert
    assert_eq!(line, "partition_update shutdown");
    Launcher::stop_all(vec![launched]).expect("failed to stop process");
}

#[test]
fn broadcast_partition_and_heal_three_processes_all_receive_recovery_signals() {
    // Arrange
    let mut launched = spawn_echo_processes(3);

    // Act
    Launcher::broadcast_partition_command(&mut launched, 1, 2)
        .expect("failed to broadcast partition command");
    let mut partition_lines = Vec::new();
    for node in &mut launched {
        partition_lines.push(
            Launcher::read_stdout_line(node)
                .expect("failed to read partition line")
                .expect("expected partition line"),
        );
    }

    Launcher::broadcast_heal_command(&mut launched, 1, 2)
        .expect("failed to broadcast heal command");
    let mut heal_lines = Vec::new();
    for node in &mut launched {
        heal_lines.push(
            Launcher::read_stdout_line(node)
                .expect("failed to read heal line")
                .expect("expected heal line"),
        );
    }

    // Assert
    for line in partition_lines {
        assert_eq!(line, "partition_update partition 1 2");
    }
    for line in heal_lines {
        assert_eq!(line, "partition_update heal 1 2");
    }
    Launcher::stop_all(launched).expect("failed to stop processes");
}

#[test]
fn broadcast_partition_and_heal_real_node_processes_emit_partition_logs() {
    // Arrange
    let binary_path = node_process_binary_path();
    let config_path = create_three_node_cluster_config();
    env::set_var("ETHERAM_NODE_PROCESS_BIN", &binary_path);
    env::set_var("ETHERAM_DESKTOP_NODE_STEP_LIMIT", "0");
    env::set_var("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND", "grpc");

    let config = ClusterConfig::load_from_path(&config_path).expect("failed to load config");
    let config_path_text = config_path.to_string_lossy().to_string();
    let mut launched = Launcher::spawn_node_processes(&config, &config_path_text)
        .expect("failed to spawn node-process children");

    thread::sleep(Duration::from_millis(300));

    // Act
    let test_result = (|| -> Result<(), String> {
        Launcher::broadcast_partition_command(&mut launched, 1, 2)?;
        for node in &mut launched {
            read_until_contains(node, "partition_update blocked from_peer=1 to_peer=2", 16)?;
        }

        Launcher::broadcast_heal_command(&mut launched, 1, 2)?;
        for node in &mut launched {
            read_until_contains(node, "partition_update healed from_peer=1 to_peer=2", 16)?;
        }

        Ok(())
    })();

    let stop_result = Launcher::stop_all(launched);

    let _ = fs::remove_file(config_path);
    env::remove_var("ETHERAM_NODE_PROCESS_BIN");
    env::remove_var("ETHERAM_DESKTOP_NODE_STEP_LIMIT");
    env::remove_var("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND");

    // Assert
    assert!(
        stop_result.is_ok(),
        "failed to stop launched nodes: {:?}",
        stop_result
    );
    assert!(
        test_result.is_ok(),
        "real node-process partition/heal validation failed: {:?}",
        test_result
    );
}
