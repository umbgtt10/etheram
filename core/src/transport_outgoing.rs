// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::types::PeerId;

pub trait TransportOutgoing {
    type Message;

    fn send(&self, peer_id: PeerId, message: Self::Message);
}
