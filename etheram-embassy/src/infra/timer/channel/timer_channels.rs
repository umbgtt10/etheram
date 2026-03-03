// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use crate::config::TIMER_COMMAND_CAPACITY;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use etheram::incoming::timer::timer_event::TimerEvent;

type TimerCommandChannel = Channel<CriticalSectionRawMutex, TimerEvent, TIMER_COMMAND_CAPACITY>;

pub static TIMER_CHANNELS: [TimerCommandChannel; MAX_NODES] = [
    Channel::new(),
    Channel::new(),
    Channel::new(),
    Channel::new(),
    Channel::new(),
];
