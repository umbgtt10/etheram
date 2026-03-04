// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::RaftMessage;
use crate::brain::protocol::message_source::MessageSource;
use crate::brain::protocol::unified_message::Message;
use crate::common_types::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use crate::common_types::timer_input_adapter::TimerInputAdapter;
use crate::common_types::transport_incoming_adapter::TransportIncomingAdapter;
use crate::incoming::external_interface::client_request::RaftClientRequest;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::boxed::Box;

pub struct RaftIncomingSources<P> {
    timer: Box<dyn TimerInputAdapter<RaftTimerEvent>>,
    external_interface: Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>,
    transport: Box<dyn TransportIncomingAdapter<RaftMessage<P>>>,
}

impl<P: 'static> RaftIncomingSources<P> {
    pub fn new(
        timer: Box<dyn TimerInputAdapter<RaftTimerEvent>>,
        external_interface: Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>>,
        transport: Box<dyn TransportIncomingAdapter<RaftMessage<P>>>,
    ) -> Self {
        Self {
            timer,
            external_interface,
            transport,
        }
    }

    pub fn poll(&self) -> Option<(MessageSource, Message<P>)> {
        if let Some((peer_id, message)) = self.transport.poll() {
            return Some((MessageSource::Peer(peer_id), Message::Peer(message)));
        }
        if let Some(event) = self.timer.poll() {
            return Some((MessageSource::Timer, Message::Timer(event)));
        }
        if let Some((client_id, request)) = self.external_interface.poll_request() {
            return Some((MessageSource::Client(client_id), Message::Client(request)));
        }
        None
    }
}
