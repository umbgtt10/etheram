// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::TIMER_COMMAND_CAPACITY;
use crate::embassy_shared_state::EmbassySharedState;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_sync::channel::Sender;
use etheram_core::node_common::shared_state::SharedState;
use raft_node::common_types::node_role::NodeRole;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

pub type TimerSender =
    Sender<'static, CriticalSectionRawMutex, RaftTimerEvent, TIMER_COMMAND_CAPACITY>;

pub type TimerReceiver =
    Receiver<'static, CriticalSectionRawMutex, RaftTimerEvent, TIMER_COMMAND_CAPACITY>;

pub struct SpawnedNode {
    pub(crate) timer_sender: TimerSender,
    pub(crate) commit_index: EmbassySharedState<u64>,
    pub(crate) term: EmbassySharedState<u64>,
    pub(crate) role: EmbassySharedState<NodeRole>,
}

impl SpawnedNode {
    pub(crate) fn read_commit_index(&self) -> u64 {
        self.commit_index.with(|c| *c)
    }

    pub(crate) fn read_term(&self) -> u64 {
        self.term.with(|t| *t)
    }

    pub(crate) fn read_role(&self) -> NodeRole {
        self.role.with(|r| *r)
    }
}
