// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::brain::protocol::action::Action;
use crate::executor::outgoing::outgoing_sources::OutgoingSources;
use alloc::vec::Vec;
use etheram_core::collection::Collection;
use etheram_core::types::PeerId;

pub struct EtheramExecutor<M> {
    outgoing_sources: OutgoingSources<M>,
    peers: Vec<PeerId>,
}

impl<M: Clone + 'static> EtheramExecutor<M> {
    pub fn new(outgoing_sources: OutgoingSources<M>) -> Self {
        Self {
            outgoing_sources,
            peers: Vec::new(),
        }
    }

    pub fn new_with_peers(outgoing_sources: OutgoingSources<M>, peers: Vec<PeerId>) -> Self {
        Self {
            outgoing_sources,
            peers,
        }
    }

    pub fn execute_outputs<A>(&self, actions: &A)
    where
        A: Collection<Item = Action<M>>,
    {
        for action in actions.iter() {
            match action {
                Action::SendMessage { to, message } => {
                    self.outgoing_sources.send_message(*to, message.clone());
                }
                Action::BroadcastMessage { message } => {
                    for peer_id in &self.peers {
                        self.outgoing_sources
                            .send_message(*peer_id, message.clone());
                    }
                }
                Action::SendClientResponse {
                    client_id,
                    response,
                } => {
                    self.outgoing_sources
                        .send_client_response(*client_id, response.clone());
                }
                Action::ScheduleTimeout { event, delay } => {
                    self.outgoing_sources.schedule_timeout(*event, *delay);
                }
                Action::Log { message: _ } => {}
                Action::UpdateAccount {
                    address: _,
                    account: _,
                } => {}
                Action::IncrementHeight => {}
                Action::StoreBlock { block: _ } => {}
                Action::ExecuteBlock { block: _ } => {}
                Action::UpdateCache { update: _ } => {}
            }
        }
    }
}
