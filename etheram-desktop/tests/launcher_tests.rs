// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_desktop::cluster_config::NodeConfig;
use etheram_desktop::launcher::Launcher;

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
