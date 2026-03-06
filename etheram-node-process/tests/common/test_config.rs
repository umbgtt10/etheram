// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn create_test_config() -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();
    path.push(format!("etheram_node_process_test_{}.toml", millis));

    let config = r#"[fleet]
validator_set = [1, 2, 3, 4, 5]
log_level = "info"

[[node]]
id = 1
transport_addr = "127.0.0.1:7001"
client_addr = "127.0.0.1:8001"
db_path = "./data/node1"
"#;

    fs::write(&path, config).expect("failed to write temporary config file");
    path
}
