// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::format;
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

impl<P> SemihostingRaftStorage<P> {
    pub fn new(node_id: PeerId) -> Self {
        Self {
            node_id,
            mutation_count: 0,
            current_term: 0,
            voted_for: None,
            log: BTreeMap::new(),
            snapshot: None,
        }
    }
}

impl<P: AsRef<[u8]> + core::fmt::Debug> SemihostingRaftStorage<P> {
    fn metadata_path(&self) -> Vec<u8> {
        format!("persistency/raft_node_{}_metadata.bin\0", self.node_id).into_bytes()
    }

    fn log_path(&self) -> Vec<u8> {
        format!("persistency/raft_node_{}_log.bin\0", self.node_id).into_bytes()
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
        let _ = self.write_file(&self.metadata_path(), &data);
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
        let _ = self.write_file(&self.log_path(), &data);
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
}

impl<P: Clone + AsRef<[u8]> + core::fmt::Debug + 'static> Storage for SemihostingRaftStorage<P> {
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
            }
        }
    }
}
