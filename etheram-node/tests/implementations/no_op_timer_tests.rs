// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::timer_input::TimerInput;
use etheram_core::timer_output::TimerOutput;
use etheram_node::implementations::no_op_timer::NoOpTimer;
use etheram_node::incoming::timer::timer_event::TimerEvent;

#[test]
fn poll_always_returns_none() {
    // Arrange
    let timer = NoOpTimer;

    // Act & Assert
    assert!(timer.poll().is_none());
}

#[test]
fn schedule_does_not_panic() {
    // Arrange
    let timer = NoOpTimer;

    // Act & Assert
    timer.schedule(TimerEvent::ProposeBlock, 1_000);
}

#[test]
fn clone_produces_independent_instance() {
    // Arrange
    let timer = NoOpTimer;

    // Act
    let cloned = timer.clone();

    // Assert
    assert!(cloned.poll().is_none());
}
