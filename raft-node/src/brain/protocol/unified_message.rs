// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::RaftMessage;
use crate::incoming::external_interface::client_request::RaftClientRequest;
use crate::incoming::timer::timer_event::RaftTimerEvent;

#[derive(Debug, Clone)]
pub enum Message<P> {
    Client(RaftClientRequest),
    Peer(RaftMessage<P>),
    Timer(RaftTimerEvent),
}
