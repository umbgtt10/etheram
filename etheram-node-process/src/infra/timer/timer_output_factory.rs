// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::timer_output_adapter::TimerOutputAdapter;
use etheram_node::builders::timer_output_builder::TimerOutputBuilder;
use etheram_node::incoming::timer::timer_event::TimerEvent;

pub fn build_timer_output() -> Result<Box<dyn TimerOutputAdapter<TimerEvent, u64>>, String> {
    TimerOutputBuilder::default()
        .build()
        .map_err(|error| format!("failed to build timer output: {error:?}"))
}
