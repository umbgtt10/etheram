// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::timer_input::TimerInput;

pub trait TimerInputAdapter<E>: TimerInput<Event = E> {}

impl<T, E> TimerInputAdapter<E> for T where T: TimerInput<Event = E> {}
