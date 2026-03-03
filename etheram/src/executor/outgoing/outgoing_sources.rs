// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use crate::common_types::timer_output_adapter::TimerOutputAdapter;
use crate::common_types::transport_outgoing_adapter::TransportOutputAdapter;
use crate::executor::outgoing::external_interface::client_response::ClientResponse;
use crate::incoming::timer::timer_event::TimerEvent;
use alloc::boxed::Box;

pub struct OutgoingSources<M> {
    timer: Box<dyn TimerOutputAdapter<TimerEvent, u64>>,
    external_interface: Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>,
    transport: Box<dyn TransportOutputAdapter<M>>,
}

impl<M: 'static> OutgoingSources<M> {
    pub fn new(
        timer: Box<dyn TimerOutputAdapter<TimerEvent, u64>>,
        external_interface: Box<dyn ExternalInterfaceOutgoingAdapter<ClientResponse>>,
        transport: Box<dyn TransportOutputAdapter<M>>,
    ) -> Self {
        Self {
            timer,
            external_interface,
            transport,
        }
    }
    pub fn schedule_timeout(&self, event: TimerEvent, delay: u64) {
        self.timer.schedule(event, delay);
    }
    pub fn send_client_response(&self, client_id: u64, response: ClientResponse) {
        self.external_interface.send_response(client_id, response);
    }
    pub fn send_message(&self, peer_id: u64, message: M) {
        self.transport.send(peer_id, message);
    }
}
