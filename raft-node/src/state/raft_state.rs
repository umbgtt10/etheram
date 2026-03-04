// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::collections::action_collection::ActionCollection;
use crate::common_types::cache_adapter::CacheAdapter;
use crate::common_types::log_entry::LogEntry;
use crate::common_types::node_role::NodeRole;
use crate::common_types::storage_adapter::StorageAdapter;
use crate::state::cache::cache_query::RaftCacheQuery;
use crate::state::cache::cache_query_result::RaftCacheQueryResult;
use crate::state::cache::cache_update::RaftCacheUpdate;
use crate::state::storage::storage_mutation::RaftStorageMutation;
use crate::state::storage::storage_query::RaftStorageQuery;
use crate::state::storage::storage_query_result::RaftStorageQueryResult;
use alloc::boxed::Box;
use alloc::vec::Vec;
use etheram_core::collection::Collection;
use etheram_core::types::PeerId;

pub struct RaftState<P> {
    storage: Box<dyn StorageAdapter<P, Key = (), Value = ()>>,
    cache: Box<dyn CacheAdapter<Key = (), Value = ()>>,
}

impl<P: Clone + 'static> RaftState<P> {
    pub fn new(
        storage: Box<dyn StorageAdapter<P, Key = (), Value = ()>>,
        cache: Box<dyn CacheAdapter<Key = (), Value = ()>>,
    ) -> Self {
        Self { storage, cache }
    }

    pub fn query_current_term(&self) -> u64 {
        match self.storage.query(RaftStorageQuery::CurrentTerm) {
            RaftStorageQueryResult::CurrentTerm(t) => t,
            _ => 0,
        }
    }

    pub fn query_voted_for(&self) -> Option<PeerId> {
        match self.storage.query(RaftStorageQuery::VotedFor) {
            RaftStorageQueryResult::VotedFor(v) => v,
            _ => None,
        }
    }

    pub fn query_log_length(&self) -> u64 {
        match self.storage.query(RaftStorageQuery::LogLength) {
            RaftStorageQueryResult::LogLength(l) => l,
            _ => 0,
        }
    }

    pub fn query_last_log_term(&self) -> u64 {
        match self.storage.query(RaftStorageQuery::LastLogTerm) {
            RaftStorageQueryResult::LastLogTerm(t) => t,
            _ => 0,
        }
    }

    pub fn query_all_entries(&self) -> Vec<LogEntry<P>> {
        match self.storage.query(RaftStorageQuery::AllEntries) {
            RaftStorageQueryResult::AllEntries(entries) => entries,
            _ => Vec::new(),
        }
    }

    pub fn query_commit_index(&self) -> u64 {
        match self.cache.query(RaftCacheQuery::CommitIndex) {
            RaftCacheQueryResult::CommitIndex(i) => i,
            _ => 0,
        }
    }

    pub fn query_last_applied(&self) -> u64 {
        match self.cache.query(RaftCacheQuery::LastApplied) {
            RaftCacheQueryResult::LastApplied(i) => i,
            _ => 0,
        }
    }

    pub fn query_role(&self) -> NodeRole {
        match self.cache.query(RaftCacheQuery::Role) {
            RaftCacheQueryResult::Role(r) => r,
            _ => crate::common_types::node_role::NodeRole::Follower,
        }
    }

    pub fn query_leader_id(&self) -> Option<PeerId> {
        match self.cache.query(RaftCacheQuery::LeaderId) {
            RaftCacheQueryResult::LeaderId(id) => id,
            _ => None,
        }
    }

    pub fn query_snapshot(&self) -> Option<crate::common_types::snapshot::RaftSnapshot> {
        match self.storage.query(RaftStorageQuery::Snapshot) {
            RaftStorageQueryResult::Snapshot(s) => s,
            _ => None,
        }
    }

    pub fn query_all_match_index(&self) -> alloc::collections::BTreeMap<PeerId, u64> {
        match self.cache.query(RaftCacheQuery::AllMatchIndex) {
            RaftCacheQueryResult::AllMatchIndex(m) => m,
            _ => alloc::collections::BTreeMap::new(),
        }
    }

    pub fn query_all_next_index(&self) -> alloc::collections::BTreeMap<PeerId, u64> {
        match self.cache.query(RaftCacheQuery::AllNextIndex) {
            RaftCacheQueryResult::AllNextIndex(n) => n,
            _ => alloc::collections::BTreeMap::new(),
        }
    }

    pub fn set_last_applied(&mut self, index: u64) {
        self.cache.update(RaftCacheUpdate::SetLastApplied(index));
    }

    pub fn apply_mutations(&mut self, mutations: &ActionCollection<RaftAction<P>>) {
        for action in mutations.iter() {
            match action {
                RaftAction::SetTerm(term) => {
                    self.storage.mutate(RaftStorageMutation::SetTerm(*term));
                }
                RaftAction::SetVotedFor(peer) => {
                    self.storage.mutate(RaftStorageMutation::SetVotedFor(*peer));
                }
                RaftAction::AppendEntries(entries) => {
                    self.storage
                        .mutate(RaftStorageMutation::AppendEntries(entries.clone()));
                }
                RaftAction::TruncateLogFrom(index) => {
                    self.storage
                        .mutate(RaftStorageMutation::TruncateFrom(*index));
                }
                RaftAction::SaveSnapshot(snapshot) => {
                    self.storage
                        .mutate(RaftStorageMutation::SaveSnapshot(snapshot.clone()));
                }
                RaftAction::AdvanceCommitIndex(index) => {
                    self.cache.update(RaftCacheUpdate::SetCommitIndex(*index));
                }
                RaftAction::TransitionRole(role) => {
                    self.cache.update(RaftCacheUpdate::SetRole(*role));
                }
                RaftAction::SetLeaderId(id) => {
                    self.cache.update(RaftCacheUpdate::SetLeaderId(*id));
                }
                RaftAction::UpdateMatchIndex { peer_id, index } => {
                    self.cache
                        .update(RaftCacheUpdate::UpdateMatchIndex(*peer_id, *index));
                }
                RaftAction::UpdateNextIndex { peer_id, index } => {
                    self.cache
                        .update(RaftCacheUpdate::UpdateNextIndex(*peer_id, *index));
                }
                _ => {}
            }
        }
    }
}
