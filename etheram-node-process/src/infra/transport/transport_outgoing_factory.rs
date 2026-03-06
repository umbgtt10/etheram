// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;
use etheram_core::transport_outgoing::TransportOutgoing;
use etheram_core::types::PeerId;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;

struct LocalNoOpOutgoingTransport;

impl TransportOutgoing for LocalNoOpOutgoingTransport {
    type Message = IbftMessage;

    fn send(&self, _peer_id: PeerId, _message: Self::Message) {}
}

pub fn build_transport_outgoing() -> Result<Box<dyn TransportOutgoingAdapter<IbftMessage>>, String>
{
    Ok(Box::new(LocalNoOpOutgoingTransport))
}
