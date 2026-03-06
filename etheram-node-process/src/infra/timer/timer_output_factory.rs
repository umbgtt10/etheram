// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::std_shared_state::StdSharedState;
use etheram_core::node_common::timer_output_adapter::TimerOutputAdapter;
use etheram_core::types::PeerId;
use etheram_node::implementations::in_memory_timer::InMemoryTimer;
use etheram_node::implementations::in_memory_timer::InMemoryTimerState;
use etheram_node::incoming::timer::timer_event::TimerEvent;

pub fn build_timer_output(
    peer_id: PeerId,
    timer_state: StdSharedState<InMemoryTimerState>,
) -> Result<Box<dyn TimerOutputAdapter<TimerEvent, u64>>, String> {
    Ok(Box::new(InMemoryTimer::new(peer_id, timer_state)))
}
