// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use alloc::boxed::Box;
use raft_node::common_types::timer_input_adapter::TimerInputAdapter;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

pub struct RaftTimerInputBuilder {
    timer: Option<Box<dyn TimerInputAdapter<RaftTimerEvent>>>,
}

impl RaftTimerInputBuilder {
    pub fn new() -> Self {
        Self { timer: None }
    }

    pub fn with_timer(mut self, timer: Box<dyn TimerInputAdapter<RaftTimerEvent>>) -> Self {
        self.timer = Some(timer);
        self
    }

    pub fn build(self) -> Result<Box<dyn TimerInputAdapter<RaftTimerEvent>>, BuildError> {
        self.timer
            .ok_or(BuildError::MissingComponent("timer_input"))
    }
}

impl Default for RaftTimerInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}
