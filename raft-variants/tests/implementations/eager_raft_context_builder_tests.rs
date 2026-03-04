// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::cache::Cache;
use etheram_core::storage::Storage;
use raft_node::brain::protocol::message_source::MessageSource;
use raft_node::brain::protocol::unified_message::Message;
use raft_node::common_types::node_role::NodeRole;
use raft_node::context::context_builder::RaftContextBuilder;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_node::state::cache::cache_update::RaftCacheUpdate;
use raft_node::state::raft_state::RaftState;
use raft_node::state::storage::storage_mutation::RaftStorageMutation;
use raft_variants::implementations::eager_raft_context_builder::EagerRaftContextBuilder;
use raft_variants::implementations::in_memory_raft_cache::InMemoryRaftCache;
use raft_variants::implementations::in_memory_raft_storage::InMemoryRaftStorage;

#[test]
fn build_reads_current_state_into_context() {
    // Arrange
    let mut storage = InMemoryRaftStorage::<Vec<u8>>::new();
    storage.mutate(RaftStorageMutation::SetTerm(7));
    storage.mutate(RaftStorageMutation::SetVotedFor(Some(3)));

    let mut cache = InMemoryRaftCache::new();
    cache.update(RaftCacheUpdate::SetRole(NodeRole::Leader));
    cache.update(RaftCacheUpdate::SetCommitIndex(5));
    cache.update(RaftCacheUpdate::SetLastApplied(4));
    cache.update(RaftCacheUpdate::SetLeaderId(Some(1)));

    let state = RaftState::new(Box::new(storage), Box::new(cache));
    let builder = EagerRaftContextBuilder::new(1, vec![2, 3, 4]);

    // Act
    let context = builder.build(
        &state,
        999,
        &MessageSource::Timer,
        &Message::Timer(RaftTimerEvent::Heartbeat),
    );

    // Assert
    assert_eq!(context.peer_id, 1);
    assert_eq!(context.current_term, 7);
    assert_eq!(context.voted_for, Some(3));
    assert_eq!(context.role, NodeRole::Leader);
    assert_eq!(context.commit_index, 5);
    assert_eq!(context.last_applied, 4);
    assert_eq!(context.leader_id, Some(1));
    assert_eq!(context.peers, vec![2, 3, 4]);
}

#[test]
fn build_returns_cloned_peer_list() {
    // Arrange
    let state = RaftState::new(
        Box::new(InMemoryRaftStorage::<Vec<u8>>::new()),
        Box::new(InMemoryRaftCache::new()),
    );
    let builder = EagerRaftContextBuilder::new(1, vec![2, 3, 4]);

    // Act
    let mut context = builder.build(
        &state,
        1,
        &MessageSource::Client(1),
        &Message::Client(RaftClientRequest::Query("k".into())),
    );
    context.peers.clear();
    let context_again = builder.build(
        &state,
        1,
        &MessageSource::Client(1),
        &Message::Client(RaftClientRequest::Query("k".into())),
    );

    // Assert
    assert_eq!(context_again.peers, vec![2, 3, 4]);
}
