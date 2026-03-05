// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer_output::TimerOutput;

pub trait TimerOutputAdapter<E, D>: TimerOutput<Event = E, Duration = D> {}

impl<T, E, D> TimerOutputAdapter<E, D> for T where T: TimerOutput<Event = E, Duration = D> {}
