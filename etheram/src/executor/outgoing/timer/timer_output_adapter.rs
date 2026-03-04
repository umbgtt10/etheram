// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::timer_output_adapter::TimerOutputAdapter;
use alloc::boxed::Box;
use etheram_core::timer_output::TimerOutput;

impl<E, D> TimerOutput for Box<dyn TimerOutputAdapter<E, D>>
where
    E: 'static,
    D: 'static,
{
    type Event = E;
    type Duration = D;

    fn schedule(&self, event: Self::Event, delay: Self::Duration) {
        (**self).schedule(event, delay)
    }
}
