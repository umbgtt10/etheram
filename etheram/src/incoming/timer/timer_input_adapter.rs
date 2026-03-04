// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::timer_input_adapter::TimerInputAdapter;
use alloc::boxed::Box;
use etheram_core::timer_input::TimerInput;

impl<E> TimerInput for Box<dyn TimerInputAdapter<E>>
where
    E: 'static,
{
    type Event = E;

    fn poll(&self) -> Option<Self::Event> {
        (**self).poll()
    }
}
