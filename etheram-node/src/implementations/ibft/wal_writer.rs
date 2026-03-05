// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::ibft::consensus_wal::ConsensusWal;
use alloc::boxed::Box;

pub trait WalWriter {
    fn write(&mut self, wal: &ConsensusWal);
}

pub struct NoOpWalWriter;

impl WalWriter for NoOpWalWriter {
    fn write(&mut self, _wal: &ConsensusWal) {}
}

pub struct CompositeWalWriter {
    first: Box<dyn WalWriter>,
    second: Box<dyn WalWriter>,
}

impl CompositeWalWriter {
    pub fn new(first: Box<dyn WalWriter>, second: Box<dyn WalWriter>) -> Self {
        Self { first, second }
    }
}

impl WalWriter for CompositeWalWriter {
    fn write(&mut self, wal: &ConsensusWal) {
        self.first.write(wal);
        self.second.write(wal);
    }
}
