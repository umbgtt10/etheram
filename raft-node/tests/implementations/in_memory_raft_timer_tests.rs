// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::standard_shared_state::StdSharedState;
use etheram_core::timer_input::TimerInput;
use etheram_core::timer_output::TimerOutput;
use raft_node::implementations::in_memory_raft_timer::InMemoryRaftTimer;
use raft_node::implementations::in_memory_raft_timer::InMemoryRaftTimerState;
use raft_node::implementations::shared_state::SharedState;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

#[test]
fn poll_empty_queue_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTimerState::new());
    let timer = InMemoryRaftTimer::new(1, state);

    // Act
    let event = timer.poll();

    // Assert
    assert!(event.is_none());
}

#[test]
fn push_event_then_poll_returns_event() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTimerState::new());
    let timer = InMemoryRaftTimer::new(1, state.clone());
    state.with_mut(|s| s.push_event(1, RaftTimerEvent::ElectionTimeout));

    // Act
    let event = timer.poll();

    // Assert
    assert!(matches!(event, Some(RaftTimerEvent::ElectionTimeout)));
}

#[test]
fn push_multiple_events_poll_drains_all_then_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTimerState::new());
    let timer = InMemoryRaftTimer::new(1, state.clone());
    state.with_mut(|s| s.push_event(1, RaftTimerEvent::ElectionTimeout));
    state.with_mut(|s| s.push_event(1, RaftTimerEvent::Heartbeat));

    // Act
    let first = timer.poll();
    let second = timer.poll();
    let third = timer.poll();

    // Assert
    assert!(matches!(first, Some(RaftTimerEvent::ElectionTimeout)));
    assert!(matches!(second, Some(RaftTimerEvent::Heartbeat)));
    assert!(third.is_none());
}

#[test]
fn schedule_event_does_not_enqueue_event() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTimerState::new());
    let timer = InMemoryRaftTimer::new(1, state);

    // Act
    timer.schedule(RaftTimerEvent::Heartbeat, 100);
    let event = timer.poll();

    // Assert
    assert!(event.is_none());
}

#[test]
fn push_event_different_node_id_not_visible_to_other_node() {
    // Arrange
    let state = StdSharedState::new(InMemoryRaftTimerState::new());
    let timer_a = InMemoryRaftTimer::new(1, state.clone());
    let _timer_b = InMemoryRaftTimer::new(2, state.clone());
    state.with_mut(|s| s.push_event(2, RaftTimerEvent::ElectionTimeout));

    // Act
    let result = timer_a.poll();

    // Assert
    assert!(result.is_none());
}
