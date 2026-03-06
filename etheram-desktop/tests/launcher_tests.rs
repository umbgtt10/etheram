// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_desktop::cluster_config::NodeConfig;
use etheram_desktop::launcher::Launcher;

#[test]
fn send_partition_command_powershell_echo_process_returns_partition_update_line() {
    // Arrange
    let program = "powershell".to_string();
    let args = vec![
        "-NoProfile".to_string(),
        "-Command".to_string(),
        "while (($line = [Console]::In.ReadLine()) -ne $null) { Write-Output \"partition_update $line\"; break }"
            .to_string(),
    ];
    let node = NodeConfig {
        id: 1,
        transport_addr: "127.0.0.1:7001".to_string(),
        client_addr: "127.0.0.1:8001".to_string(),
        db_path: "./data/node1".to_string(),
    };
    let mut launched = Launcher::spawn_node_with_command(&program, &args, &node)
        .expect("failed to spawn echo process");

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
