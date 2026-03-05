// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::log_entry::LogEntry;
use crate::common_types::snapshot::RaftSnapshot;
use crate::state::storage::storage_mutation::RaftStorageMutation;
use crate::state::storage::storage_query::RaftStorageQuery;
use crate::state::storage::storage_query_result::RaftStorageQueryResult;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::storage::Storage;
use etheram_core::types::PeerId;

pub struct InMemoryRaftStorage<P> {
    current_term: u64,
    voted_for: Option<PeerId>,
    log: BTreeMap<u64, LogEntry<P>>,
    snapshot: Option<RaftSnapshot>,
}

impl<P> InMemoryRaftStorage<P> {
    pub fn new() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: BTreeMap::new(),
            snapshot: None,
        }
    }
}

impl<P> Default for InMemoryRaftStorage<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Clone + 'static> Storage for InMemoryRaftStorage<P> {
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
        match mutation {
            RaftStorageMutation::SetTerm(term) => {
                self.current_term = term;
            }
            RaftStorageMutation::SetVotedFor(peer) => {
                self.voted_for = peer;
            }
            RaftStorageMutation::AppendEntries(entries) => {
                for entry in entries {
                    self.log.insert(entry.index, entry);
                }
            }
            RaftStorageMutation::TruncateFrom(idx) => {
                self.log.retain(|&k, _| k < idx);
            }
            RaftStorageMutation::SaveSnapshot(snap) => {
                self.log.retain(|&k, _| k > snap.last_included_index);
                self.snapshot = Some(snap);
            }
        }
    }
}
