// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::format;
use alloc::vec::Vec;
use etheram_core::types::PeerId;
use etheram_etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_etheram_variants::implementations::ibft::wal_writer::WalWriter;

pub struct SemihostingWalWriter {
    node_id: PeerId,
}

impl SemihostingWalWriter {
    pub fn new(node_id: PeerId) -> Self {
        Self { node_id }
    }
}

impl WalWriter for SemihostingWalWriter {
    fn write(&mut self, wal: &ConsensusWal) {
        let path = wal_path(self.node_id);
        let data = wal.to_bytes();
        let _ = write_file(&path, &data);
    }
}

fn wal_path(node_id: PeerId) -> Vec<u8> {
    format!("persistency/etheram_node_{}_wal.bin\0", node_id).into_bytes()
}

fn write_file(path: &[u8], data: &[u8]) -> Result<(), ()> {
    unsafe {
        let mode: usize = 0x0000_0004;
        let fd = cortex_m_semihosting::syscall!(OPEN, path.as_ptr(), mode, path.len() - 1);
        if fd == usize::MAX {
            return Err(());
        }
        let result = cortex_m_semihosting::syscall!(WRITE, fd, data.as_ptr(), data.len());
        cortex_m_semihosting::syscall!(CLOSE, fd);
        if result == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
