// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::transport_incoming_adapter::TransportInputAdapter;
use alloc::boxed::Box;
use etheram_core::{transport_incoming::TransportIncoming, types::PeerId};

impl<Msg> TransportIncoming for Box<dyn TransportInputAdapter<Msg>>
where
    Msg: 'static,
{
    type Message = Msg;

    fn poll(&self) -> Option<(PeerId, Self::Message)> {
        (**self).poll()
    }
}
