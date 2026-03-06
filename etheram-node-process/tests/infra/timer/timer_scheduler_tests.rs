// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::timer_input::TimerInput;
use etheram_node::implementations::in_memory_timer::InMemoryTimer;
use etheram_node::implementations::in_memory_timer::InMemoryTimerState;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node_process::infra::std_shared_state::StdSharedState;
use etheram_node_process::infra::timer::timer_scheduler::TimerScheduler;
use std::thread;
use std::time::Duration;

fn drain_events(timer: &InMemoryTimer<StdSharedState<InMemoryTimerState>>) -> Vec<TimerEvent> {
    let mut events = Vec::new();
    while let Some(event) = timer.poll() {
        events.push(event);
    }
    events
}

#[test]
fn tick_before_propose_interval_does_not_push_propose_event() {
    // Arrange
    let node_id = 1;
    let timer_state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(node_id, timer_state.clone());
    let mut scheduler = TimerScheduler::new(timer_state);

    // Act
    scheduler.tick(node_id);

    // Assert
    let events = drain_events(&timer);
    assert!(events.is_empty());
}

#[test]
fn tick_after_propose_interval_pushes_propose_block_event() {
    // Arrange
    let node_id = 2;
    let timer_state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(node_id, timer_state.clone());
    let mut scheduler = TimerScheduler::new(timer_state);
    thread::sleep(Duration::from_millis(260));

    // Act
    scheduler.tick(node_id);

    // Assert
    let events = drain_events(&timer);
    assert!(events.contains(&TimerEvent::ProposeBlock));
    assert!(!events.contains(&TimerEvent::TimeoutRound));
}

#[test]
fn tick_after_timeout_interval_pushes_timeout_round_event() {
    // Arrange
    let node_id = 3;
    let timer_state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(node_id, timer_state.clone());
    let mut scheduler = TimerScheduler::new(timer_state);
    thread::sleep(Duration::from_millis(1510));

    // Act
    scheduler.tick(node_id);

    // Assert
    let events = drain_events(&timer);
    assert!(events.contains(&TimerEvent::TimeoutRound));
}

#[test]
fn tick_after_both_intervals_pushes_both_events() {
    // Arrange
    let node_id = 4;
    let timer_state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(node_id, timer_state.clone());
    let mut scheduler = TimerScheduler::new(timer_state);
    thread::sleep(Duration::from_millis(1510));

    // Act
    scheduler.tick(node_id);

    // Assert
    let events = drain_events(&timer);
    assert!(events.contains(&TimerEvent::ProposeBlock));
    assert!(events.contains(&TimerEvent::TimeoutRound));
}

#[test]
fn tick_resets_propose_timer_after_pushing_event() {
    // Arrange
    let node_id = 5;
    let timer_state = StdSharedState::new(InMemoryTimerState::new());
    let timer = InMemoryTimer::new(node_id, timer_state.clone());
    let mut scheduler = TimerScheduler::new(timer_state);
    thread::sleep(Duration::from_millis(260));
    scheduler.tick(node_id);
    let first_events = drain_events(&timer);
    assert!(first_events.contains(&TimerEvent::ProposeBlock));

    // Act
    scheduler.tick(node_id);

    // Assert
    let second_events = drain_events(&timer);
    assert!(second_events.is_empty());
}
