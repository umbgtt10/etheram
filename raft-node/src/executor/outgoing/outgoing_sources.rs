// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::message::RaftMessage;
use crate::common_types::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use crate::common_types::timer_output_adapter::TimerOutputAdapter;
use crate::common_types::transport_outgoing_adapter::TransportOutgoingAdapter;
use crate::executor::outgoing::external_interface::client_response::RaftClientResponse;
use crate::incoming::timer::timer_event::RaftTimerEvent;
use alloc::boxed::Box;
use etheram_core::types::{ClientId, PeerId};

pub struct RaftOutgoingSources<P> {
    timer: Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>,
    external_interface: Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>,
    transport: Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>,
}

impl<P: 'static> RaftOutgoingSources<P> {
    pub fn new(
        timer: Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>>,
        external_interface: Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>>,
        transport: Box<dyn TransportOutgoingAdapter<RaftMessage<P>>>,
    ) -> Self {
        Self {
            timer,
            external_interface,
            transport,
        }
    }

    pub fn send_message(&self, to: PeerId, message: RaftMessage<P>) {
        self.transport.send(to, message);
    }

    pub fn schedule_timeout(&self, event: RaftTimerEvent, delay: u64) {
        self.timer.schedule(event, delay);
    }

    pub fn send_client_response(&self, client_id: ClientId, response: RaftClientResponse) {
        self.external_interface.send_response(client_id, response);
    }
}
