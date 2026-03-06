// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::partitionable_transport::partition_table::global_partition_table;
use etheram_core::types::PeerId;
use std::io;
use std::io::BufRead;
use std::thread;

enum PartitionControlCommand {
    Partition,
    Heal,
    Clear,
}

impl PartitionControlCommand {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "partition" => Some(Self::Partition),
            "heal" => Some(Self::Heal),
            "clear" => Some(Self::Clear),
            _ => None,
        }
    }
}

pub fn spawn_partition_control_thread() -> Result<(), String> {
    let builder = thread::Builder::new().name("partition-control".to_string());
    builder
        .spawn(move || {
            let stdin = io::stdin();
            let lines = stdin.lock().lines();
            for line_result in lines {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if let Err(error) = apply_control_line(&line) {
                            println!("partition_control_error {}", error);
                        }
                    }
                    Err(error) => {
                        println!("partition_control_error failed to read stdin: {}", error);
                        break;
                    }
                }
            }
        })
        .map(|_| ())
        .map_err(|error| format!("failed to spawn partition control thread: {error}"))
}

fn apply_control_line(line: &str) -> Result<(), String> {
    let mut parts = line.split_whitespace();
    let Some(raw_command) = parts.next() else {
        return Ok(());
    };
    let command = PartitionControlCommand::parse(raw_command).ok_or_else(|| {
        format!(
            "unknown command '{}', expected partition|heal|clear",
            raw_command
        )
    })?;

    match command {
        PartitionControlCommand::Partition => {
            let from = parse_peer(parts.next(), "from")?;
            let to = parse_peer(parts.next(), "to")?;
            global_partition_table().block(from, to);
            println!("partition_update blocked from_peer={} to_peer={}", from, to);
            Ok(())
        }
        PartitionControlCommand::Heal => {
            let from = parse_peer(parts.next(), "from")?;
            let to = parse_peer(parts.next(), "to")?;
            global_partition_table().heal(from, to);
            println!("partition_update healed from_peer={} to_peer={}", from, to);
            Ok(())
        }
        PartitionControlCommand::Clear => {
            global_partition_table().clear();
            println!("partition_update cleared");
            Ok(())
        }
    }
}

fn parse_peer(raw: Option<&str>, field: &str) -> Result<PeerId, String> {
    let value = raw.ok_or_else(|| format!("missing {} peer id", field))?;
    value
        .parse::<PeerId>()
        .map_err(|error| format!("invalid {} peer id '{}': {}", field, value, error))
}
