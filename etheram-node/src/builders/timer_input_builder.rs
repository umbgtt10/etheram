// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::common_types::timer_input_adapter::TimerInputAdapter;
use crate::implementations::no_op_timer::NoOpTimer;
use crate::incoming::timer::timer_event::TimerEvent;
use crate::variants::TimerInputVariant;
use alloc::boxed::Box;

pub struct TimerInputBuilder {
    timer: Option<Box<dyn TimerInputAdapter<TimerEvent>>>,
}

impl TimerInputBuilder {
    pub fn new() -> Self {
        Self { timer: None }
    }

    pub fn with_variant(mut self, variant: TimerInputVariant) -> Self {
        let timer = match variant {
            TimerInputVariant::NoOp => Box::new(NoOpTimer),
            TimerInputVariant::Custom(custom) => custom,
        };
        self.timer = Some(timer);
        self
    }

    pub fn build(self) -> Result<Box<dyn TimerInputAdapter<TimerEvent>>, BuildError> {
        self.timer
            .ok_or(BuildError::MissingComponent("timer_input"))
    }
}

impl Default for TimerInputBuilder {
    fn default() -> Self {
        Self {
            timer: Some(Box::new(NoOpTimer)),
        }
    }
}
