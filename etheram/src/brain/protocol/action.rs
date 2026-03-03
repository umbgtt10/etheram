// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{
    common_types::{account::Account, block::Block, types::Address},
    executor::outgoing::external_interface::client_response::ClientResponse,
    incoming::timer::timer_event::TimerEvent,
    state::cache::cache_update::CacheUpdate,
};
use alloc::string::String;
use barechain_core::types::{ClientId, PeerId};

#[derive(Debug, Clone)]
pub enum Action<M> {
    BroadcastMessage {
        message: M,
    },
    SendMessage {
        to: PeerId,
        message: M,
    },

    SendClientResponse {
        client_id: ClientId,
        response: ClientResponse,
    },

    UpdateAccount {
        address: Address,
        account: Account,
    },

    IncrementHeight,

    StoreBlock {
        block: Block,
    },

    ExecuteBlock {
        block: Block,
    },

    UpdateCache {
        update: CacheUpdate,
    },

    ScheduleTimeout {
        event: TimerEvent,
        delay: u64,
    },

    Log {
        message: String,
    },
}
