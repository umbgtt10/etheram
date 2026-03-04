// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

pub trait RaftStateMachine {
    fn apply_raw(&mut self, index: u64, command: &[u8]) -> Vec<u8>;
    fn query_raw(&self, key: &[u8]) -> Vec<u8>;
    fn snapshot(&self) -> Vec<u8>;
    fn restore(&mut self, data: &[u8]);
}
