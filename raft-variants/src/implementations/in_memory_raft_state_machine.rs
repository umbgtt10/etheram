// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use raft_node::common_types::state_machine::RaftStateMachine;

pub struct InMemoryRaftStateMachine {
    store: BTreeMap<String, Vec<u8>>,
    applied_count: u64,
}

impl InMemoryRaftStateMachine {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
            applied_count: 0,
        }
    }

    pub fn applied_count(&self) -> u64 {
        self.applied_count
    }
}

impl Default for InMemoryRaftStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl RaftStateMachine for InMemoryRaftStateMachine {
    fn apply_raw(&mut self, _index: u64, command: &[u8]) -> Vec<u8> {
        self.applied_count += 1;
        if command.len() < 2 {
            return Vec::new();
        }
        let key_len = command[0] as usize;
        if command.len() < 1 + key_len {
            return Vec::new();
        }
        let key = String::from_utf8_lossy(&command[1..1 + key_len]).into_owned();
        let value = &command[1 + key_len..];
        self.store.insert(key, value.to_vec());
        Vec::new()
    }

    fn query_raw(&self, key: &[u8]) -> Vec<u8> {
        let key_str = String::from_utf8_lossy(key).into_owned();
        self.store.get(&key_str).cloned().unwrap_or_default()
    }

    fn snapshot(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for (k, v) in &self.store {
            let kb = k.as_bytes();
            result.push(kb.len() as u8);
            result.extend_from_slice(kb);
            result.extend_from_slice(&(v.len() as u16).to_le_bytes());
            result.extend_from_slice(v);
        }
        result
    }

    fn restore(&mut self, data: &[u8]) {
        self.store.clear();
        let mut i = 0;
        while i < data.len() {
            if i >= data.len() {
                break;
            }
            let key_len = data[i] as usize;
            i += 1;
            if i + key_len > data.len() {
                break;
            }
            let key = String::from_utf8_lossy(&data[i..i + key_len]).into_owned();
            i += key_len;
            if i + 2 > data.len() {
                break;
            }
            let val_len = u16::from_le_bytes([data[i], data[i + 1]]) as usize;
            i += 2;
            if i + val_len > data.len() {
                break;
            }
            let value = data[i..i + val_len].to_vec();
            i += val_len;
            self.store.insert(key, value);
        }
    }
}
