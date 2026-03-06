// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_desktop::ui::cluster_state::log_visible;
use etheram_desktop::ui::cluster_state::ClusterState;
use etheram_desktop::ui::cluster_state::ConvergenceStatus;

#[test]
fn apply_node_status_from_line_valid_line_inserts_snapshot() {
    // Arrange
    let mut state = ClusterState::new();

    // Act
    state.apply_node_status_from_line("node_status peer_id=1 height=42 last_hash=abc123");

    // Assert
    assert_eq!(state.latest_node_status.len(), 1);
    let snapshot = state.latest_node_status.get(&1).unwrap();
    assert_eq!(snapshot.height, 42);
    assert_eq!(snapshot.last_hash, "abc123");
}

#[test]
fn apply_node_status_from_line_missing_field_does_nothing() {
    // Arrange
    let mut state = ClusterState::new();

    // Act
    state.apply_node_status_from_line("node_status peer_id=1 height=42");

    // Assert
    assert!(state.latest_node_status.is_empty());
}

#[test]
fn apply_node_status_from_line_no_marker_does_nothing() {
    // Arrange
    let mut state = ClusterState::new();

    // Act
    state.apply_node_status_from_line("some random log line peer_id=1 height=42 last_hash=abc");

    // Assert
    assert!(state.latest_node_status.is_empty());
}

#[test]
fn apply_node_status_from_line_updates_existing_peer() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_node_status_from_line("node_status peer_id=1 height=10 last_hash=old");

    // Act
    state.apply_node_status_from_line("node_status peer_id=1 height=20 last_hash=new");

    // Assert
    assert_eq!(state.latest_node_status.len(), 1);
    let snapshot = state.latest_node_status.get(&1).unwrap();
    assert_eq!(snapshot.height, 20);
    assert_eq!(snapshot.last_hash, "new");
}

#[test]
fn apply_partition_update_from_line_blocked_adds_link() {
    // Arrange
    let mut state = ClusterState::new();

    // Act
    state.apply_partition_update_from_line("partition_update blocked from_peer=1 to_peer=2");

    // Assert
    assert!(state.blocked_links.contains(&(1, 2)));
    assert_eq!(state.blocked_links.len(), 1);
}

#[test]
fn apply_partition_update_from_line_healed_removes_link() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_partition_update_from_line("partition_update blocked from_peer=1 to_peer=2");

    // Act
    state.apply_partition_update_from_line("partition_update healed from_peer=1 to_peer=2");

    // Assert
    assert!(state.blocked_links.is_empty());
}

#[test]
fn apply_partition_update_from_line_cleared_removes_all_links() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_partition_update_from_line("partition_update blocked from_peer=1 to_peer=2");
    state.apply_partition_update_from_line("partition_update blocked from_peer=3 to_peer=4");

    // Act
    state.apply_partition_update_from_line("partition_update cleared");

    // Assert
    assert!(state.blocked_links.is_empty());
}

#[test]
fn apply_partition_update_from_line_no_marker_does_nothing() {
    // Arrange
    let mut state = ClusterState::new();

    // Act
    state.apply_partition_update_from_line("some random log from_peer=1 to_peer=2");

    // Assert
    assert!(state.blocked_links.is_empty());
}

#[test]
fn apply_partition_update_from_line_missing_peers_does_nothing() {
    // Arrange
    let mut state = ClusterState::new();

    // Act
    state.apply_partition_update_from_line("partition_update blocked from_peer=1");

    // Assert
    assert!(state.blocked_links.is_empty());
}

#[test]
fn convergence_status_zero_nodes_returns_stopped() {
    // Arrange
    let state = ClusterState::new();

    // Act
    let result = state.convergence_status(0);

    // Assert
    assert!(matches!(result, ConvergenceStatus::Stopped));
    assert!(result.label().contains("cluster stopped"));
}

#[test]
fn convergence_status_incomplete_returns_waiting() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_node_status_from_line("node_status peer_id=1 height=10 last_hash=aaa");

    // Act
    let result = state.convergence_status(3);

    // Assert
    assert!(matches!(
        result,
        ConvergenceStatus::Waiting { have: 1, want: 3 }
    ));
    assert!(result.label().contains("1/3"));
}

#[test]
fn convergence_status_all_same_returns_converged() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_node_status_from_line("node_status peer_id=1 height=10 last_hash=abc");
    state.apply_node_status_from_line("node_status peer_id=2 height=10 last_hash=abc");
    state.apply_node_status_from_line("node_status peer_id=3 height=10 last_hash=abc");

    // Act
    let result = state.convergence_status(3);

    // Assert
    assert!(matches!(
        result,
        ConvergenceStatus::Converged { height: 10, .. }
    ));
    assert!(result.label().contains("converged"));
}

#[test]
fn convergence_status_different_heights_returns_diverged() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_node_status_from_line("node_status peer_id=1 height=10 last_hash=abc");
    state.apply_node_status_from_line("node_status peer_id=2 height=15 last_hash=def");

    // Act
    let result = state.convergence_status(2);

    // Assert
    assert!(matches!(result, ConvergenceStatus::Diverged { .. }));
    assert!(result.label().contains("diverged"));
}

#[test]
fn convergence_status_same_height_different_hash_returns_diverged() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_node_status_from_line("node_status peer_id=1 height=10 last_hash=abc");
    state.apply_node_status_from_line("node_status peer_id=2 height=10 last_hash=xyz");

    // Act
    let result = state.convergence_status(2);

    // Assert
    assert!(matches!(result, ConvergenceStatus::Diverged { .. }));
}

#[test]
fn log_visible_no_filter_returns_true() {
    // Act & Assert
    assert!(log_visible("", "", 1, "some log line"));
}

#[test]
fn log_visible_node_filter_matches_returns_true() {
    // Act & Assert
    assert!(log_visible("1", "", 1, "some log line"));
}

#[test]
fn log_visible_node_filter_no_match_returns_false() {
    // Act & Assert
    assert!(!log_visible("2", "", 1, "some log line"));
}

#[test]
fn log_visible_text_filter_matches_returns_true() {
    // Act & Assert
    assert!(log_visible("", "error", 1, "an Error occurred"));
}

#[test]
fn log_visible_text_filter_no_match_returns_false() {
    // Act & Assert
    assert!(!log_visible("", "warning", 1, "an error occurred"));
}

#[test]
fn log_visible_invalid_node_filter_returns_false() {
    // Act & Assert
    assert!(!log_visible("abc", "", 1, "some log line"));
}

#[test]
fn log_visible_both_filters_applied() {
    // Act & Assert
    assert!(log_visible("1", "error", 1, "an error occurred"));
    assert!(!log_visible("2", "error", 1, "an error occurred"));
    assert!(!log_visible("1", "warning", 1, "an error occurred"));
}

#[test]
fn clear_resets_all_state() {
    // Arrange
    let mut state = ClusterState::new();
    state.apply_node_status_from_line("node_status peer_id=1 height=10 last_hash=abc");
    state.apply_partition_update_from_line("partition_update blocked from_peer=1 to_peer=2");

    // Act
    state.clear();

    // Assert
    assert!(state.latest_node_status.is_empty());
    assert!(state.blocked_links.is_empty());
}
