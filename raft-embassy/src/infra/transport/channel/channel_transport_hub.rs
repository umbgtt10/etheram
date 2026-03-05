// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use alloc::vec::Vec;
use embassy_core::channel_transport_hub::GenericChannelTransportHub;
use raft_node::brain::protocol::message::RaftMessage;

const TRANSPORT_CAPACITY: usize = 64;

type TransportPayload = Vec<u8>;

pub type ChannelTransportHub =
    GenericChannelTransportHub<RaftMessage<TransportPayload>, MAX_NODES, TRANSPORT_CAPACITY>;

pub static TRANSPORT_HUB: ChannelTransportHub = ChannelTransportHub::new();
