// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::cache::Cache;
use etheram_core::types::PeerId;
use raft_node::common_types::node_role::NodeRole;
use raft_node::state::cache::cache_query::RaftCacheQuery;
use raft_node::state::cache::cache_query_result::RaftCacheQueryResult;
use raft_node::state::cache::cache_update::RaftCacheUpdate;

pub struct InMemoryRaftCache {
    commit_index: u64,
    last_applied: u64,
    role: NodeRole,
    leader_id: Option<PeerId>,
    match_index: BTreeMap<PeerId, u64>,
    next_index: BTreeMap<PeerId, u64>,
}

impl InMemoryRaftCache {
    pub fn new() -> Self {
        Self {
            commit_index: 0,
            last_applied: 0,
            role: NodeRole::Follower,
            leader_id: None,
            match_index: BTreeMap::new(),
            next_index: BTreeMap::new(),
        }
    }
}

impl Default for InMemoryRaftCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for InMemoryRaftCache {
    type Key = ();
    type Value = ();
    type Query = RaftCacheQuery;
    type Update = RaftCacheUpdate;
    type QueryResult = RaftCacheQueryResult;

    fn query(&self, query: Self::Query) -> Self::QueryResult {
        match query {
            RaftCacheQuery::CommitIndex => RaftCacheQueryResult::CommitIndex(self.commit_index),
            RaftCacheQuery::LastApplied => RaftCacheQueryResult::LastApplied(self.last_applied),
            RaftCacheQuery::Role => RaftCacheQueryResult::Role(self.role),
            RaftCacheQuery::LeaderId => RaftCacheQueryResult::LeaderId(self.leader_id),
            RaftCacheQuery::MatchIndex(peer) => {
                RaftCacheQueryResult::MatchIndex(self.match_index.get(&peer).copied().unwrap_or(0))
            }
            RaftCacheQuery::NextIndex(peer) => {
                RaftCacheQueryResult::NextIndex(self.next_index.get(&peer).copied().unwrap_or(1))
            }
            RaftCacheQuery::AllMatchIndex => {
                RaftCacheQueryResult::AllMatchIndex(self.match_index.clone())
            }
            RaftCacheQuery::AllNextIndex => {
                RaftCacheQueryResult::AllNextIndex(self.next_index.clone())
            }
        }
    }

    fn update(&mut self, update: Self::Update) {
        match update {
            RaftCacheUpdate::SetCommitIndex(idx) => {
                self.commit_index = idx;
            }
            RaftCacheUpdate::SetLastApplied(idx) => {
                self.last_applied = idx;
            }
            RaftCacheUpdate::SetRole(role) => {
                self.role = role;
            }
            RaftCacheUpdate::SetLeaderId(id) => {
                self.leader_id = id;
            }
            RaftCacheUpdate::UpdateMatchIndex(peer, idx) => {
                self.match_index.insert(peer, idx);
            }
            RaftCacheUpdate::UpdateNextIndex(peer, idx) => {
                self.next_index.insert(peer, idx);
            }
            RaftCacheUpdate::InitPeers(peers) => {
                for peer in peers {
                    self.match_index.entry(peer).or_insert(0);
                    self.next_index.entry(peer).or_insert(1);
                }
            }
        }
    }

    fn invalidate(&mut self, _key: Self::Key) {}
}

impl InMemoryRaftCache {
    pub fn init_peers_with_next_index(&mut self, peers: Vec<PeerId>, next_index: u64) {
        for peer in peers {
            self.match_index.insert(peer, 0);
            self.next_index.insert(peer, next_index);
        }
    }
}
