// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::builders::error::BuildError;
use crate::implementations::in_memory_raft_timer::InMemoryRaftTimer;
use crate::implementations::in_memory_raft_timer::InMemoryRaftTimerState;
use crate::implementations::shared_state::SharedState;
use crate::variants::RaftTimerInputVariant;
use alloc::boxed::Box;
use etheram_core::types::PeerId;
use raft_node::common_types::timer_input_adapter::TimerInputAdapter;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

pub struct RaftTimerInputBuilder<S: SharedState<InMemoryRaftTimerState> + 'static> {
    timer: Option<Box<dyn TimerInputAdapter<RaftTimerEvent>>>,
    shared_state: Option<S>,
    peer_id: Option<PeerId>,
}

impl<S: SharedState<InMemoryRaftTimerState> + 'static> RaftTimerInputBuilder<S> {
    pub fn new() -> Self {
        Self {
            timer: None,
            shared_state: None,
            peer_id: None,
        }
    }

    pub fn with_shared_state(mut self, shared_state: S) -> Self {
        self.shared_state = Some(shared_state);
        self
    }

    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn with_variant(mut self, variant: RaftTimerInputVariant) -> Self {
        match variant {
            RaftTimerInputVariant::Manual => {
                let peer_id = self.peer_id.expect("PeerId required for Manual timer");
                let state = self
                    .shared_state
                    .clone()
                    .expect("SharedState required for Manual timer");
                self.timer = Some(Box::new(InMemoryRaftTimer::new(peer_id, state)));
            }
            RaftTimerInputVariant::Custom(custom) => {
                self.timer = Some(custom);
            }
        }
        self
    }

    pub fn build(self) -> Result<Box<dyn TimerInputAdapter<RaftTimerEvent>>, BuildError> {
        self.timer
            .ok_or(BuildError::MissingComponent("timer_input"))
    }
}
