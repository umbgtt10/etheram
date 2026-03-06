// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn next_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to allocate local port");
    listener
        .local_addr()
        .expect("failed to get local socket address")
        .port()
}

pub fn create_test_config() -> PathBuf {
    let mut path = std::env::temp_dir();
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "etheram_node_process_test_{}_{}_{}.toml",
        std::process::id(),
        nanos,
        unique
    ));
    let transport_port = next_port();
    let client_port = next_port();

    let config = format!(
        r#"[fleet]
validator_set = [1, 2, 3, 4, 5]
log_level = "info"

[[node]]
id = 1
transport_addr = "127.0.0.1:{transport_port}"
client_addr = "127.0.0.1:{client_port}"
db_path = "./data/node1"
"#
    );

    fs::write(&path, config).expect("failed to write temporary config file");
    path
}
