// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::shared_state::SharedState;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::{timer_input::TimerInput, timer_output::TimerOutput};
use etheram_node::incoming::timer::timer_event::TimerEvent;

pub struct InMemoryTimerState {
    events: BTreeMap<u64, Vec<TimerEvent>>,
}

impl InMemoryTimerState {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn push_event(&mut self, node_id: u64, event: TimerEvent) {
        self.events.entry(node_id).or_default().push(event);
    }
}

impl Default for InMemoryTimerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct InMemoryTimer<S>
where
    S: SharedState<InMemoryTimerState>,
{
    node_id: u64,
    state: S,
}

impl<S> InMemoryTimer<S>
where
    S: SharedState<InMemoryTimerState>,
{
    pub fn new(node_id: u64, state: S) -> Self {
        state.with_mut(|state| {
            state.events.insert(node_id, Vec::new());
        });
        Self { node_id, state }
    }
}

impl<S> TimerInput for InMemoryTimer<S>
where
    S: SharedState<InMemoryTimerState>,
{
    type Event = TimerEvent;

    fn poll(&self) -> Option<Self::Event> {
        self.state.with_mut(|state| {
            if let Some(queue) = state.events.get_mut(&self.node_id) {
                if !queue.is_empty() {
                    return Some(queue.remove(0));
                }
            }
            None
        })
    }
}

impl<S> TimerOutput for InMemoryTimer<S>
where
    S: SharedState<InMemoryTimerState>,
{
    type Event = TimerEvent;
    type Duration = u64;

    fn schedule(&self, event: Self::Event, _delay: Self::Duration) {
        self.state.with_mut(|state| {
            state.events.entry(self.node_id).or_default().push(event);
        });
    }
}
