// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::no_op_timer::NoOpTimer;
use crate::incoming::timer::timer_event::TimerEvent;
use crate::variants::TimerOutputVariant;
use alloc::boxed::Box;
use etheram_core::node_common::timer_output_adapter::TimerOutputAdapter;

pub struct TimerOutputBuilder {
    timer: Option<Box<dyn TimerOutputAdapter<TimerEvent, u64>>>,
}

impl TimerOutputBuilder {
    pub fn new() -> Self {
        Self { timer: None }
    }

    pub fn with_variant(mut self, variant: TimerOutputVariant) -> Self {
        let timer = match variant {
            TimerOutputVariant::NoOp => Box::new(NoOpTimer),
            TimerOutputVariant::Custom(custom) => custom,
        };
        self.timer = Some(timer);
        self
    }

    pub fn build(self) -> Result<Box<dyn TimerOutputAdapter<TimerEvent, u64>>, BuildError> {
        self.timer
            .ok_or(BuildError::MissingComponent("timer_output"))
    }
}

impl Default for TimerOutputBuilder {
    fn default() -> Self {
        Self {
            timer: Some(Box::new(NoOpTimer)),
        }
    }
}
