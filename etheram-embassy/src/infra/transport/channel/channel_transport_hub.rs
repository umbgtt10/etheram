// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::MAX_NODES;
use embassy_core::channel_transport_hub::GenericChannelTransportHub;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;

const TRANSPORT_CAPACITY: usize = 64;

pub type ChannelTransportHub =
    GenericChannelTransportHub<IbftMessage, MAX_NODES, TRANSPORT_CAPACITY>;

pub static TRANSPORT_HUB: ChannelTransportHub = ChannelTransportHub::new();
