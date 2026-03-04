// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::{ClientId, PeerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSource {
    Peer(PeerId),
    Client(ClientId),
    Timer,
}
