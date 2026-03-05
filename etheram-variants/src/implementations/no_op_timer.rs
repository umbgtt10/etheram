// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::{timer_input::TimerInput, timer_output::TimerOutput};
use etheram_node::incoming::timer::timer_event::TimerEvent;

#[derive(Clone)]
pub struct NoOpTimer;

impl TimerInput for NoOpTimer {
    type Event = TimerEvent;

    fn poll(&self) -> Option<Self::Event> {
        None
    }
}

impl TimerOutput for NoOpTimer {
    type Event = TimerEvent;
    type Duration = u64;

    fn schedule(&self, _event: Self::Event, _delay: Self::Duration) {}
}
