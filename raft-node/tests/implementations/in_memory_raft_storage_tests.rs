// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::storage::Storage;
use raft_node::common_types::log_entry::LogEntry;
use raft_node::common_types::snapshot::RaftSnapshot;
use raft_node::implementations::in_memory_raft_storage::InMemoryRaftStorage;
use raft_node::state::storage::storage_mutation::RaftStorageMutation;
use raft_node::state::storage::storage_query::RaftStorageQuery;
use raft_node::state::storage::storage_query_result::RaftStorageQueryResult;

fn make_storage() -> InMemoryRaftStorage<Vec<u8>> {
    InMemoryRaftStorage::new()
}

#[test]
fn query_log_length_empty_storage_returns_zero() {
    // Arrange
    let storage = make_storage();

    // Act
    let result = storage.query(RaftStorageQuery::LogLength);

    // Assert
    assert!(matches!(result, RaftStorageQueryResult::LogLength(0)));
}

#[test]
fn append_entries_then_query_log_length_returns_last_index() {
    // Arrange
    let mut storage = make_storage();
    let entries = vec![
        LogEntry {
            term: 1,
            index: 1,
            payload: vec![],
        },
        LogEntry {
            term: 1,
            index: 2,
            payload: vec![],
        },
    ];

    // Act
    storage.mutate(RaftStorageMutation::AppendEntries(entries));
    let result = storage.query(RaftStorageQuery::LogLength);

    // Assert
    assert!(matches!(result, RaftStorageQueryResult::LogLength(2)));
}

#[test]
fn query_all_entries_returns_appended_entries() {
    // Arrange
    let mut storage = make_storage();
    let entries = vec![LogEntry {
        term: 1,
        index: 1,
        payload: vec![42u8],
    }];

    // Act
    storage.mutate(RaftStorageMutation::AppendEntries(entries));
    let result = storage.query(RaftStorageQuery::AllEntries);

    // Assert
    if let RaftStorageQueryResult::AllEntries(all) = result {
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].index, 1);
    } else {
        panic!("unexpected result");
    }
}

#[test]
fn query_entry_at_index_returns_correct_entry() {
    // Arrange
    let mut storage = make_storage();
    storage.mutate(RaftStorageMutation::AppendEntries(vec![
        LogEntry {
            term: 1,
            index: 1,
            payload: vec![1u8],
        },
        LogEntry {
            term: 1,
            index: 2,
            payload: vec![2u8],
        },
    ]));

    // Act
    let result = storage.query(RaftStorageQuery::EntryAt(1));

    // Assert
    if let RaftStorageQueryResult::Entry(Some(e)) = result {
        assert_eq!(e.index, 1);
        assert_eq!(e.payload, vec![1u8]);
    } else {
        panic!("unexpected result");
    }
}

#[test]
fn truncate_from_removes_entries_at_and_after_index() {
    // Arrange
    let mut storage = make_storage();
    storage.mutate(RaftStorageMutation::AppendEntries(vec![
        LogEntry {
            term: 1,
            index: 1,
            payload: vec![],
        },
        LogEntry {
            term: 1,
            index: 2,
            payload: vec![],
        },
        LogEntry {
            term: 1,
            index: 3,
            payload: vec![],
        },
    ]));

    // Act
    storage.mutate(RaftStorageMutation::TruncateFrom(2));
    let result = storage.query(RaftStorageQuery::AllEntries);

    // Assert
    if let RaftStorageQueryResult::AllEntries(all) = result {
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].index, 1);
    } else {
        panic!("unexpected result");
    }
}

#[test]
fn save_snapshot_then_query_returns_snapshot() {
    // Arrange
    let mut storage = make_storage();
    let snapshot = RaftSnapshot {
        last_included_index: 5,
        last_included_term: 2,
        data: vec![7u8],
    };

    // Act
    storage.mutate(RaftStorageMutation::SaveSnapshot(snapshot));
    let result = storage.query(RaftStorageQuery::Snapshot);

    // Assert
    if let RaftStorageQueryResult::Snapshot(Some(s)) = result {
        assert_eq!(s.last_included_index, 5);
        assert_eq!(s.last_included_term, 2);
    } else {
        panic!("unexpected result");
    }
}

#[test]
fn save_snapshot_truncates_log_entries_before_snapshot_index() {
    // Arrange
    let mut storage = make_storage();
    storage.mutate(RaftStorageMutation::AppendEntries(vec![
        LogEntry {
            term: 1,
            index: 1,
            payload: vec![],
        },
        LogEntry {
            term: 1,
            index: 2,
            payload: vec![],
        },
        LogEntry {
            term: 1,
            index: 3,
            payload: vec![],
        },
    ]));
    let snapshot = RaftSnapshot {
        last_included_index: 2,
        last_included_term: 1,
        data: vec![],
    };

    // Act
    storage.mutate(RaftStorageMutation::SaveSnapshot(snapshot));
    let result = storage.query(RaftStorageQuery::AllEntries);

    // Assert
    if let RaftStorageQueryResult::AllEntries(all) = result {
        assert!(all.iter().all(|e| e.index > 2));
    } else {
        panic!("unexpected result");
    }
}
