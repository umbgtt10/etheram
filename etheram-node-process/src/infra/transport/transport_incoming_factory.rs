// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::transport_incoming_adapter::TransportIncomingAdapter;
use etheram_core::transport_incoming::TransportIncoming;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;

struct LocalNoOpIncomingTransport;

impl TransportIncoming for LocalNoOpIncomingTransport {
    type Message = IbftMessage;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        None
    }
}

pub fn build_transport_incoming() -> Result<Box<dyn TransportIncomingAdapter<IbftMessage>>, String>
{
    Ok(Box::new(LocalNoOpIncomingTransport))
}
