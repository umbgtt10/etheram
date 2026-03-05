// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::transport_outgoing_adapter::TransportOutgoingAdapter;
use alloc::boxed::Box;
use etheram_core::{transport_outgoing::TransportOutgoing, types::PeerId};

impl<Msg> TransportOutgoing for Box<dyn TransportOutgoingAdapter<Msg>>
where
    Msg: 'static,
{
    type Message = Msg;

    fn send(&self, peer_id: PeerId, message: Self::Message) {
        (**self).send(peer_id, message)
    }
}
