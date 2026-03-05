// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub type TimerCommandChannel<E, const CAPACITY: usize> =
    Channel<CriticalSectionRawMutex, E, CAPACITY>;

pub struct GenericTimerChannels<E, const N: usize, const CAPACITY: usize> {
    channels: [TimerCommandChannel<E, CAPACITY>; N],
}

impl<E, const N: usize, const CAPACITY: usize> GenericTimerChannels<E, N, CAPACITY> {
    pub const fn new() -> Self {
        Self {
            channels: [const { Channel::new() }; N],
        }
    }

    pub fn channel(&self, node_index: usize) -> &TimerCommandChannel<E, CAPACITY> {
        &self.channels[node_index]
    }
}

impl<E, const N: usize, const CAPACITY: usize> Default for GenericTimerChannels<E, N, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}
