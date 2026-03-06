// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::test_config::create_test_config;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

fn read_stderr_to_string(mut stderr: std::process::ChildStderr) -> String {
    let mut buffer = String::new();
    let _ = stderr.read_to_string(&mut buffer);
    buffer
}

#[test]
fn main_run_forever_shutdown_command_exits_successfully() {
    // Arrange
    let config_path = create_test_config();
    let mut child = Command::new(env!("CARGO_BIN_EXE_etheram-node-process"))
        .arg(&config_path)
        .arg("1")
        .env_remove("ETHERAM_PARTITION_BLOCKS")
        .env_remove("ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("failed to spawn node process: {}", error));

    thread::sleep(Duration::from_millis(100));

    // Act
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(b"shutdown\n")
            .unwrap_or_else(|error| panic!("failed to send shutdown command: {}", error));
        stdin
            .flush()
            .unwrap_or_else(|error| panic!("failed to flush shutdown command: {}", error));
    } else {
        panic!("child stdin not available");
    }

    let started = Instant::now();
    let status = loop {
        if let Some(exit_status) = child
            .try_wait()
            .unwrap_or_else(|error| panic!("failed to poll child status: {}", error))
        {
            break exit_status;
        }
        if started.elapsed() > Duration::from_secs(2) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run-forever process did not exit within timeout after shutdown");
        }
        thread::sleep(Duration::from_millis(20));
    };

    // Assert
    let stderr_output = child
        .stderr
        .take()
        .map(read_stderr_to_string)
        .unwrap_or_default();
    assert!(
        status.success(),
        "expected successful exit, got {} stderr: {}",
        status,
        stderr_output
    );
    let _ = fs::remove_file(config_path);
}
