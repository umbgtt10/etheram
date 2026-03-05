// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::{
    brain::protocol::{message::Message, message_source::MessageSource},
    common_types::{
        external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter,
        timer_input_adapter::TimerInputAdapter,
        transport_incoming_adapter::TransportIncomingAdapter,
    },
    incoming::{external_interface::client_request::ClientRequest, timer::timer_event::TimerEvent},
};
use alloc::boxed::Box;

pub struct IncomingSources<M> {
    timer: Box<dyn TimerInputAdapter<TimerEvent>>,
    external_interface: Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>,
    transport: Box<dyn TransportIncomingAdapter<M>>,
}

impl<M: 'static> IncomingSources<M> {
    pub fn new(
        timer: Box<dyn TimerInputAdapter<TimerEvent>>,
        external_interface: Box<dyn ExternalInterfaceIncomingAdapter<ClientRequest>>,
        transport: Box<dyn TransportIncomingAdapter<M>>,
    ) -> Self {
        Self {
            timer,
            external_interface,
            transport,
        }
    }
    pub fn poll(&self) -> Option<(MessageSource, Message<M>)> {
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
