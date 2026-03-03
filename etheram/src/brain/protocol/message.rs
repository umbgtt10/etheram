// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::incoming::external_interface::client_request::ClientRequest;
use crate::incoming::timer::timer_event::TimerEvent;

#[derive(Debug, Clone)]
pub enum Message<M> {
    Client(ClientRequest),
    Peer(M),
    Timer(TimerEvent),
}
