// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::std_shared_state::StdSharedState;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::types::PeerId;
use etheram_node::implementations::in_memory_timer::InMemoryTimerState;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use std::time::Duration;
use std::time::Instant;

const PROPOSE_TICK_MS: u64 = 250;
const TIMEOUT_TICK_MS: u64 = 1500;

pub struct TimerScheduler {
    last_propose_tick_at: Instant,
    last_timeout_tick_at: Instant,
    timer_state: StdSharedState<InMemoryTimerState>,
}

impl TimerScheduler {
    pub fn new(timer_state: StdSharedState<InMemoryTimerState>) -> Self {
        Self {
            last_propose_tick_at: Instant::now(),
            last_timeout_tick_at: Instant::now(),
            timer_state,
        }
    }

    pub fn tick(&mut self, node_id: PeerId) {
        if self.last_propose_tick_at.elapsed() >= Duration::from_millis(PROPOSE_TICK_MS) {
            self.timer_state
                .with_mut(|state| state.push_event(node_id, TimerEvent::ProposeBlock));
            self.last_propose_tick_at = Instant::now();
        }

        if self.last_timeout_tick_at.elapsed() >= Duration::from_millis(TIMEOUT_TICK_MS) {
            self.timer_state
                .with_mut(|state| state.push_event(node_id, TimerEvent::TimeoutRound));
            self.last_timeout_tick_at = Instant::now();
        }
    }

    pub fn timer_state(&self) -> &StdSharedState<InMemoryTimerState> {
        &self.timer_state
    }
}
