// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::types::PeerId;
use core::option::Option;

pub trait TransportIncoming {
    type Message;

    fn poll(&self) -> Option<(PeerId, Self::Message)>;
}
