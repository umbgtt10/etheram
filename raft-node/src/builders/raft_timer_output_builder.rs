// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::common_types::timer_output_adapter::TimerOutputAdapter;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::boxed::Box;

pub struct RaftTimerOutputBuilder {
    timer: Option<Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>>,
}

impl RaftTimerOutputBuilder {
    pub fn new() -> Self {
        Self { timer: None }
    }

    pub fn with_timer(mut self, timer: Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>) -> Self {
        self.timer = Some(timer);
        self
    }

    pub fn build(self) -> Result<Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>, BuildError> {
        self.timer
            .ok_or(BuildError::MissingComponent("timer_output"))
    }
}

impl Default for RaftTimerOutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}
