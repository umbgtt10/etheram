// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::standard_shared_state::StdSharedState;
use barechain_core::timer_input::TimerInput;
use barechain_core::timer_output::TimerOutput;
use barechain_etheram_variants::implementations::in_memory_timer::{
    InMemoryTimer, InMemoryTimerState,
};
use barechain_etheram_variants::implementations::shared_state::SharedState;
use etheram::incoming::timer::timer_event::TimerEvent;

#[test]
fn poll_empty_queue_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(0, state.clone());

    // Act & Assert
    assert!(timer.poll().is_none());
}

#[test]
fn push_event_then_poll_returns_event() {
    // Arrange
    let state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(0, state.clone());
    state.with_mut(|state| {
        state.push_event(0, TimerEvent::ProposeBlock);
    });

    // Act
    let result = timer.poll();

    // Assert
    assert!(matches!(result, Some(TimerEvent::ProposeBlock)));
}

#[test]
fn push_multiple_events_poll_drains_all_then_returns_none() {
    // Arrange
    let state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(0, state.clone());
    state.with_mut(|state| {
        state.push_event(0, TimerEvent::ProposeBlock);
    });
    state.with_mut(|state| {
        state.push_event(0, TimerEvent::ProposeBlock);
    });

    // Act
    let first = timer.poll();
    let second = timer.poll();
    let third = timer.poll();

    // Assert
    assert!(matches!(first, Some(TimerEvent::ProposeBlock)));
    assert!(matches!(second, Some(TimerEvent::ProposeBlock)));
    assert!(third.is_none());
}

#[test]
fn schedule_event_then_poll_returns_event() {
    // Arrange
    let state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(0, state.clone());

    // Act
    timer.schedule(TimerEvent::ProposeBlock, 0);

    // Assert
    assert!(matches!(timer.poll(), Some(TimerEvent::ProposeBlock)));
}

#[test]
fn push_event_different_node_id_not_visible_to_other_node() {
    // Arrange
    let state = StdSharedState::new(InMemoryTimerState::new());
    let timer_a = InMemoryTimer::new(0, state.clone());
    let _timer_b = InMemoryTimer::new(1, state.clone());
    state.with_mut(|state| {
        state.push_event(1, TimerEvent::ProposeBlock);
    });

    // Act
    let result = timer_a.poll();

    // Assert
    assert!(result.is_none());
}
