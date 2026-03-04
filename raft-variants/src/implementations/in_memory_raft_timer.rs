// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::implementations::shared_state::SharedState;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::timer_input::TimerInput;
use etheram_core::timer_output::TimerOutput;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

pub struct InMemoryRaftTimerState {
    events: BTreeMap<u64, Vec<RaftTimerEvent>>,
}

impl InMemoryRaftTimerState {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    pub fn push_event(&mut self, node_id: u64, event: RaftTimerEvent) {
        self.events.entry(node_id).or_default().push(event);
    }
}

impl Default for InMemoryRaftTimerState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InMemoryRaftTimer<S>
where
    S: SharedState<InMemoryRaftTimerState>,
{
    node_id: u64,
    state: S,
}

impl<S> InMemoryRaftTimer<S>
where
    S: SharedState<InMemoryRaftTimerState>,
{
    pub fn new(node_id: u64, state: S) -> Self {
        state.with_mut(|s| {
            s.events.insert(node_id, Vec::new());
        });
        Self { node_id, state }
    }
}

impl<S> TimerInput for InMemoryRaftTimer<S>
where
    S: SharedState<InMemoryRaftTimerState>,
{
    type Event = RaftTimerEvent;

    fn poll(&self) -> Option<Self::Event> {
        self.state.with_mut(|s| {
            if let Some(queue) = s.events.get_mut(&self.node_id) {
                if !queue.is_empty() {
                    return Some(queue.remove(0));
                }
            }
            None
        })
    }
}

impl<S> TimerOutput for InMemoryRaftTimer<S>
where
    S: SharedState<InMemoryRaftTimerState>,
{
    type Event = RaftTimerEvent;
    type Duration = u64;

    fn schedule(&self, _event: Self::Event, _delay: Self::Duration) {}
}
