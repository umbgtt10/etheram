// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::common_types::state_machine::RaftStateMachine;
use raft_variants::implementations::in_memory_raft_state_machine::InMemoryRaftStateMachine;

fn encode_command(key: &str, value: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    result.push(key.len() as u8);
    result.extend_from_slice(key.as_bytes());
    result.extend_from_slice(value);
    result
}

#[test]
fn apply_raw_valid_command_then_query_raw_returns_value() {
    // Arrange
    let mut state_machine = InMemoryRaftStateMachine::new();

    // Act
    state_machine.apply_raw(1, &encode_command("alpha", b"1"));
    let value = state_machine.query_raw(b"alpha");

    // Assert
    assert_eq!(value, b"1");
}

#[test]
fn apply_raw_then_applied_count_increments() {
    // Arrange
    let mut state_machine = InMemoryRaftStateMachine::new();

    // Act
    state_machine.apply_raw(1, &encode_command("k", b"v"));

    // Assert
    assert_eq!(state_machine.applied_count(), 1);
}

#[test]
fn snapshot_then_restore_preserves_values() {
    // Arrange
    let mut original = InMemoryRaftStateMachine::new();
    original.apply_raw(1, &encode_command("k1", b"v1"));
    original.apply_raw(2, &encode_command("k2", b"v2"));
    let snapshot = original.snapshot();

    // Act
    let mut restored = InMemoryRaftStateMachine::new();
    restored.restore(&snapshot);

    // Assert
    assert_eq!(restored.query_raw(b"k1"), b"v1");
    assert_eq!(restored.query_raw(b"k2"), b"v2");
}
