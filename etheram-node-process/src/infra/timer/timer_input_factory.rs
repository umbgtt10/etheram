// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::timer_input_adapter::TimerInputAdapter;
use etheram_node::builders::timer_input_builder::TimerInputBuilder;
use etheram_node::incoming::timer::timer_event::TimerEvent;

pub fn build_timer_input() -> Result<Box<dyn TimerInputAdapter<TimerEvent>>, String> {
    TimerInputBuilder::default()
        .build()
        .map_err(|error| format!("failed to build timer input: {error:?}"))
}
