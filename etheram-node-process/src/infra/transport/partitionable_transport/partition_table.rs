// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use std::collections::BTreeSet;
use std::sync::Mutex;
use std::sync::OnceLock;

const PARTITION_BLOCKS_ENV: &str = "ETHERAM_PARTITION_BLOCKS";

pub struct PartitionTable {
    blocked_links: Mutex<BTreeSet<(PeerId, PeerId)>>,
}

impl PartitionTable {
    pub fn new() -> Self {
        Self {
            blocked_links: Mutex::new(BTreeSet::new()),
        }
    }

    pub fn initialize_from_env(&self) -> Result<usize, String> {
        self.clear();
        match std::env::var(PARTITION_BLOCKS_ENV) {
            Ok(raw) => self.load_from_csv(&raw),
            Err(_) => Ok(0),
        }
    }

    pub fn block(&self, from: PeerId, to: PeerId) {
        let mut guard = self
            .blocked_links
            .lock()
            .expect("partition table lock poisoned");
        guard.insert((from, to));
    }

    pub fn clear(&self) {
        let mut guard = self
            .blocked_links
            .lock()
            .expect("partition table lock poisoned");
        guard.clear();
    }

    pub fn is_blocked(&self, from: PeerId, to: PeerId) -> bool {
        let guard = self
            .blocked_links
            .lock()
            .expect("partition table lock poisoned");
        guard.contains(&(from, to))
    }

    fn load_from_csv(&self, csv: &str) -> Result<usize, String> {
        if csv.trim().is_empty() {
            return Ok(0);
        }

        let mut loaded = 0usize;
        for token in csv.split(',') {
            let pair = token.trim();
            if pair.is_empty() {
                continue;
            }
            let Some((from_raw, to_raw)) = pair.split_once('-') else {
                return Err(format!(
                    "invalid partition pair '{}', expected <from>-<to>",
                    pair
                ));
            };
            let from = parse_peer(from_raw)?;
            let to = parse_peer(to_raw)?;
            self.block(from, to);
            loaded += 1;
        }
        Ok(loaded)
    }
}

fn parse_peer(raw: &str) -> Result<PeerId, String> {
    raw.trim()
        .parse::<PeerId>()
        .map_err(|error| format!("invalid peer id '{}': {error}", raw.trim()))
}

pub fn global_partition_table() -> &'static PartitionTable {
    static GLOBAL_PARTITION_TABLE: OnceLock<PartitionTable> = OnceLock::new();
    GLOBAL_PARTITION_TABLE.get_or_init(PartitionTable::new)
}
