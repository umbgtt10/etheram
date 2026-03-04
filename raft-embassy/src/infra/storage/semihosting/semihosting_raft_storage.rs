// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use etheram_core::storage::Storage;
use etheram_core::types::PeerId;
use raft_node::common_types::log_entry::LogEntry;
use raft_node::common_types::snapshot::RaftSnapshot;
use raft_node::state::storage::storage_mutation::RaftStorageMutation;
use raft_node::state::storage::storage_query::RaftStorageQuery;
use raft_node::state::storage::storage_query_result::RaftStorageQueryResult;

pub struct SemihostingRaftStorage<P> {
    node_id: PeerId,
    mutation_count: u64,
    current_term: u64,
    voted_for: Option<PeerId>,
    log: BTreeMap<u64, LogEntry<P>>,
    snapshot: Option<RaftSnapshot>,
}

impl<P: Clone + From<Vec<u8>> + AsRef<[u8]> + core::fmt::Debug> SemihostingRaftStorage<P> {
    pub fn new(node_id: PeerId) -> Self {
        let mut storage = Self {
            node_id,
            mutation_count: 0,
            current_term: 0,
            voted_for: None,
            log: BTreeMap::new(),
            snapshot: None,
        };
        storage.load_from_disk();
        storage
    }
}

impl<P: AsRef<[u8]> + core::fmt::Debug> SemihostingRaftStorage<P> {
    fn metadata_path(&self) -> Vec<u8> {
        format!("persistency/raft_node_{}_metadata.bin\0", self.node_id).into_bytes()
    }

    fn log_path(&self) -> Vec<u8> {
        format!("persistency/raft_node_{}_log.bin\0", self.node_id).into_bytes()
    }

    fn snapshot_meta_path(&self) -> Vec<u8> {
        format!("persistency/raft_node_{}_snapshot_meta.bin\0", self.node_id).into_bytes()
    }

    fn snapshot_data_path(&self) -> Vec<u8> {
        format!("persistency/raft_node_{}_snapshot_data.bin\0", self.node_id).into_bytes()
    }

    fn persist_metadata(&self) {
        let mut data: Vec<u8> = Vec::with_capacity(17);
        data.extend_from_slice(&self.current_term.to_le_bytes());
        match self.voted_for {
            None => {
                data.push(0u8);
                data.extend_from_slice(&0u64.to_le_bytes());
            }
            Some(peer) => {
                data.push(1u8);
                data.extend_from_slice(&peer.to_le_bytes());
            }
        }
        if self.write_file(&self.metadata_path(), &data).is_err() {
            crate::info!(
                "raft_storage metadata persist failed for node {}",
                self.node_id
            );
        }
    }

    fn persist_log(&self) {
        let mut data: Vec<u8> = Vec::new();
        let count = self.log.len() as u64;
        data.extend_from_slice(&count.to_le_bytes());
        for entry in self.log.values() {
            data.extend_from_slice(&entry.index.to_le_bytes());
            data.extend_from_slice(&entry.term.to_le_bytes());
            let payload_bytes = entry.payload.as_ref();
            data.extend_from_slice(&(payload_bytes.len() as u32).to_le_bytes());
            data.extend_from_slice(payload_bytes);
        }
        if self.write_file(&self.log_path(), &data).is_err() {
            crate::info!("raft_storage log persist failed for node {}", self.node_id);
        }
    }

    fn persist_snapshot(&self) {
        if let Some(snapshot) = &self.snapshot {
            let mut meta_data: Vec<u8> = Vec::with_capacity(16);
            meta_data.extend_from_slice(&snapshot.last_included_index.to_le_bytes());
            meta_data.extend_from_slice(&snapshot.last_included_term.to_le_bytes());

            if self
                .write_file(&self.snapshot_meta_path(), &meta_data)
                .is_err()
            {
                crate::info!(
                    "raft_storage snapshot meta persist failed for node {}",
                    self.node_id
                );
            }

            if self
                .write_file(&self.snapshot_data_path(), &snapshot.data)
                .is_err()
            {
                crate::info!(
                    "raft_storage snapshot data persist failed for node {}",
                    self.node_id
                );
            }
        }
    }

    fn write_file(&self, path: &[u8], data: &[u8]) -> Result<(), ()> {
        unsafe {
            let mode: usize = 0x0000_0006;
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

    fn read_file(&self, path: &[u8]) -> Result<Vec<u8>, ()> {
        unsafe {
            let mode: usize = 0x0000_0000;
            let fd = cortex_m_semihosting::syscall!(OPEN, path.as_ptr(), mode, path.len() - 1);
            if fd == usize::MAX {
                return Err(());
            }

            let len = cortex_m_semihosting::syscall!(FLEN, fd);
            if len == usize::MAX {
                cortex_m_semihosting::syscall!(CLOSE, fd);
                return Err(());
            }

            let mut data = vec![0u8; len];
            let bytes_remaining = cortex_m_semihosting::syscall!(READ, fd, data.as_mut_ptr(), len);
            cortex_m_semihosting::syscall!(CLOSE, fd);

            if bytes_remaining == 0 {
                Ok(data)
            } else {
                Err(())
            }
        }
    }
}

impl<P: Clone + From<Vec<u8>> + AsRef<[u8]> + core::fmt::Debug> SemihostingRaftStorage<P> {
    fn load_from_disk(&mut self) {
        if let Ok(metadata) = self.read_file(&self.metadata_path()) {
            if metadata.len() >= 17 {
                self.current_term = u64::from_le_bytes([
                    metadata[0],
                    metadata[1],
                    metadata[2],
                    metadata[3],
                    metadata[4],
                    metadata[5],
                    metadata[6],
                    metadata[7],
                ]);
                if metadata[8] == 0 {
                    self.voted_for = None;
                } else {
                    self.voted_for = Some(u64::from_le_bytes([
                        metadata[9],
                        metadata[10],
                        metadata[11],
                        metadata[12],
                        metadata[13],
                        metadata[14],
                        metadata[15],
                        metadata[16],
                    ]));
                }
            }
        }

        if let Ok(log_data) = self.read_file(&self.log_path()) {
            self.log = self.deserialize_log(&log_data);
        }

        if let (Ok(snapshot_meta), Ok(snapshot_data)) = (
            self.read_file(&self.snapshot_meta_path()),
            self.read_file(&self.snapshot_data_path()),
        ) {
            if snapshot_meta.len() >= 16 {
                let last_included_index = u64::from_le_bytes([
                    snapshot_meta[0],
                    snapshot_meta[1],
                    snapshot_meta[2],
                    snapshot_meta[3],
                    snapshot_meta[4],
                    snapshot_meta[5],
                    snapshot_meta[6],
                    snapshot_meta[7],
                ]);
                let last_included_term = u64::from_le_bytes([
                    snapshot_meta[8],
                    snapshot_meta[9],
                    snapshot_meta[10],
                    snapshot_meta[11],
                    snapshot_meta[12],
                    snapshot_meta[13],
                    snapshot_meta[14],
                    snapshot_meta[15],
                ]);
                self.snapshot = Some(RaftSnapshot {
                    last_included_index,
                    last_included_term,
                    data: snapshot_data,
                });
            }
        }
    }

    fn deserialize_log(&self, data: &[u8]) -> BTreeMap<u64, LogEntry<P>> {
        let mut log = BTreeMap::new();
        if data.len() < 8 {
            return log;
        }

        let mut offset = 8;
        while offset + 20 <= data.len() {
            let index = u64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let term = u64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let payload_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + payload_len > data.len() {
                break;
            }

            let payload = P::from(data[offset..offset + payload_len].to_vec());
            offset += payload_len;

            log.insert(
                index,
                LogEntry {
                    index,
                    term,
                    payload,
                },
            );
        }

        log
    }
}

impl<P: Clone + From<Vec<u8>> + AsRef<[u8]> + core::fmt::Debug + 'static> Storage
    for SemihostingRaftStorage<P>
{
    type Key = ();
    type Value = ();
    type Query = RaftStorageQuery;
    type Mutation = RaftStorageMutation<P>;
    type QueryResult = RaftStorageQueryResult<P>;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        match query {
            RaftStorageQuery::CurrentTerm => RaftStorageQueryResult::CurrentTerm(self.current_term),
            RaftStorageQuery::VotedFor => RaftStorageQueryResult::VotedFor(self.voted_for),
            RaftStorageQuery::LogLength => {
                if let Some(last) = self.log.values().next_back() {
                    RaftStorageQueryResult::LogLength(last.index)
                } else if let Some(ref snap) = self.snapshot {
                    RaftStorageQueryResult::LogLength(snap.last_included_index)
                } else {
                    RaftStorageQueryResult::LogLength(0)
                }
            }
            RaftStorageQuery::LastLogTerm => {
                if let Some(last) = self.log.values().next_back() {
                    RaftStorageQueryResult::LastLogTerm(last.term)
                } else if let Some(ref snap) = self.snapshot {
                    RaftStorageQueryResult::LastLogTerm(snap.last_included_term)
                } else {
                    RaftStorageQueryResult::LastLogTerm(0)
                }
            }
            RaftStorageQuery::Snapshot => RaftStorageQueryResult::Snapshot(self.snapshot.clone()),
            RaftStorageQuery::AllEntries => {
                RaftStorageQueryResult::AllEntries(self.log.values().cloned().collect())
            }
            RaftStorageQuery::EntryAt(idx) => {
                RaftStorageQueryResult::Entry(self.log.get(&idx).cloned())
            }
            RaftStorageQuery::EntriesFrom(idx) => {
                let entries: Vec<LogEntry<P>> =
                    self.log.range(idx..).map(|(_, e)| e.clone()).collect();
                RaftStorageQueryResult::Entries(entries)
            }
        }
    }

    fn mutate(&mut self, mutation: Self::Mutation) {
        self.mutation_count += 1;
        crate::info!(
            "raft_storage mutation #{}: {:?}",
            self.mutation_count,
            mutation
        );
        match mutation {
            RaftStorageMutation::SetTerm(term) => {
                self.current_term = term;
                self.persist_metadata();
            }
            RaftStorageMutation::SetVotedFor(peer) => {
                self.voted_for = peer;
                self.persist_metadata();
            }
            RaftStorageMutation::AppendEntries(entries) => {
                for entry in entries {
                    self.log.insert(entry.index, entry);
                }
                self.persist_log();
            }
            RaftStorageMutation::TruncateFrom(idx) => {
                self.log.retain(|&k, _| k < idx);
                self.persist_log();
            }
            RaftStorageMutation::SaveSnapshot(snap) => {
                self.log.retain(|&k, _| k > snap.last_included_index);
                self.snapshot = Some(snap);
                self.persist_log();
                self.persist_snapshot();
            }
        }
    }
}
