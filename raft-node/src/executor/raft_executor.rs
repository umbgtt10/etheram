// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::RaftAction;
use crate::executor::outgoing::outgoing_sources::RaftOutgoingSources;
use alloc::vec::Vec;
use etheram_core::collection::Collection;
use etheram_core::types::PeerId;

pub struct RaftExecutor<P> {
    outgoing: RaftOutgoingSources<P>,
    peers: Vec<PeerId>,
}

impl<P: Clone + 'static> RaftExecutor<P> {
    pub fn new(outgoing: RaftOutgoingSources<P>) -> Self {
        Self {
            outgoing,
            peers: Vec::new(),
        }
    }

    pub fn new_with_peers(outgoing: RaftOutgoingSources<P>, peers: Vec<PeerId>) -> Self {
        Self { outgoing, peers }
    }

    pub fn execute_outputs<A>(&self, actions: &A)
    where
        A: Collection<Item = RaftAction<P>>,
    {
        for action in actions.iter() {
            match action {
                RaftAction::SendMessage { to, message } => {
                    self.outgoing.send_message(*to, message.clone());
                }
                RaftAction::BroadcastMessage { message } => {
                    for peer_id in &self.peers {
                        self.outgoing.send_message(*peer_id, message.clone());
                    }
                }
                RaftAction::SendClientResponse {
                    client_id,
                    response,
                } => {
                    self.outgoing
                        .send_client_response(*client_id, response.clone());
                }
                RaftAction::ScheduleTimeout { event, delay } => {
                    self.outgoing.schedule_timeout(*event, *delay);
                }
                _ => {}
            }
        }
    }
}
