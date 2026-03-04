// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::cache::Cache;
use raft_node::common_types::node_role::NodeRole;
use raft_node::state::cache::cache_query::RaftCacheQuery;
use raft_node::state::cache::cache_query_result::RaftCacheQueryResult;
use raft_node::state::cache::cache_update::RaftCacheUpdate;
use raft_variants::implementations::in_memory_raft_cache::InMemoryRaftCache;

fn make_cache() -> InMemoryRaftCache {
    InMemoryRaftCache::new()
}

#[test]
fn query_role_initial_returns_follower() {
    // Arrange
    let cache = make_cache();

    // Act
    let result = cache.query(RaftCacheQuery::Role);

    // Assert
    assert!(matches!(
        result,
        RaftCacheQueryResult::Role(NodeRole::Follower)
    ));
}

#[test]
fn set_commit_index_then_query_returns_updated_value() {
    // Arrange
    let mut cache = make_cache();

    // Act
    cache.update(RaftCacheUpdate::SetCommitIndex(7));
    let result = cache.query(RaftCacheQuery::CommitIndex);

    // Assert
    assert!(matches!(result, RaftCacheQueryResult::CommitIndex(7)));
}

#[test]
fn init_peers_then_query_next_index_returns_one() {
    // Arrange
    let mut cache = make_cache();

    // Act
    cache.update(RaftCacheUpdate::InitPeers(vec![2, 3]));
    let result = cache.query(RaftCacheQuery::NextIndex(2));

    // Assert
    assert!(matches!(result, RaftCacheQueryResult::NextIndex(1)));
}

#[test]
fn update_match_index_then_query_all_match_index_returns_updated() {
    // Arrange
    let mut cache = make_cache();
    cache.update(RaftCacheUpdate::InitPeers(vec![2, 3]));

    // Act
    cache.update(RaftCacheUpdate::UpdateMatchIndex(2, 5));
    let result = cache.query(RaftCacheQuery::AllMatchIndex);

    // Assert
    if let RaftCacheQueryResult::AllMatchIndex(map) = result {
        assert_eq!(*map.get(&2).unwrap(), 5u64);
    } else {
        panic!("unexpected result");
    }
}

#[test]
fn update_next_index_then_query_all_next_index_returns_updated() {
    // Arrange
    let mut cache = make_cache();
    cache.update(RaftCacheUpdate::InitPeers(vec![2, 3]));

    // Act
    cache.update(RaftCacheUpdate::UpdateNextIndex(3, 8));
    let result = cache.query(RaftCacheQuery::AllNextIndex);

    // Assert
    if let RaftCacheQueryResult::AllNextIndex(map) = result {
        assert_eq!(*map.get(&3).unwrap(), 8u64);
    } else {
        panic!("unexpected result");
    }
}

#[test]
fn transition_to_leader_then_query_role_returns_leader() {
    // Arrange
    let mut cache = make_cache();

    // Act
    cache.update(RaftCacheUpdate::SetRole(NodeRole::Leader));
    let result = cache.query(RaftCacheQuery::Role);

    // Assert
    assert!(matches!(
        result,
        RaftCacheQueryResult::Role(NodeRole::Leader)
    ));
}

#[test]
fn set_leader_id_then_query_returns_that_peer() {
    // Arrange
    let mut cache = make_cache();

    // Act
    cache.update(RaftCacheUpdate::SetLeaderId(Some(42)));
    let result = cache.query(RaftCacheQuery::LeaderId);

    // Assert
    assert!(matches!(result, RaftCacheQueryResult::LeaderId(Some(42))));
}
