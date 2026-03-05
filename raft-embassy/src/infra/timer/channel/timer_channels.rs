// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use crate::config::TIMER_COMMAND_CAPACITY;
use embassy_core::timer_channels::GenericTimerChannels;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

type TimerChannels = GenericTimerChannels<RaftTimerEvent, MAX_NODES, TIMER_COMMAND_CAPACITY>;

pub static TIMER_CHANNELS: TimerChannels = TimerChannels::new();
