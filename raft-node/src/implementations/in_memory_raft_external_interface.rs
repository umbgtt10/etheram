// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::executor::outgoing::external_interface::client_response::RaftClientResponse;
use crate::implementations::shared_state::SharedState;
use crate::incoming::external_interface::client_request::RaftClientRequest;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::types::ClientId;

pub struct InMemoryRaftExternalInterfaceState {
    inboxes: BTreeMap<u64, Vec<(ClientId, RaftClientRequest)>>,
    outboxes: BTreeMap<u64, Vec<(ClientId, RaftClientResponse)>>,
    responses_by_client: BTreeMap<ClientId, Vec<RaftClientResponse>>,
}

impl InMemoryRaftExternalInterfaceState {
    pub fn new() -> Self {
        Self {
            inboxes: BTreeMap::new(),
            outboxes: BTreeMap::new(),
            responses_by_client: BTreeMap::new(),
        }
    }

    pub fn push_request(&mut self, node_id: u64, client_id: ClientId, request: RaftClientRequest) {
        self.inboxes
            .entry(node_id)
            .or_default()
            .push((client_id, request));
    }

    pub fn drain_responses(&mut self, node_id: u64) -> Vec<(ClientId, RaftClientResponse)> {
        self.outboxes.remove(&node_id).unwrap_or_default()
    }

    pub fn drain_client_responses(&mut self, client_id: ClientId) -> Vec<RaftClientResponse> {
        self.responses_by_client
            .remove(&client_id)
            .unwrap_or_default()
    }
}

impl Default for InMemoryRaftExternalInterfaceState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InMemoryRaftExternalInterface<S>
where
    S: SharedState<InMemoryRaftExternalInterfaceState>,
{
    node_id: u64,
    state: S,
}

impl<S> InMemoryRaftExternalInterface<S>
where
    S: SharedState<InMemoryRaftExternalInterfaceState>,
{
    pub fn new(node_id: u64, state: S) -> Self {
        state.with_mut(|s| {
            s.inboxes.entry(node_id).or_default();
            s.outboxes.entry(node_id).or_default();
        });
        Self { node_id, state }
    }
}

impl<S> ExternalInterfaceIncoming for InMemoryRaftExternalInterface<S>
where
    S: SharedState<InMemoryRaftExternalInterfaceState>,
{
    type Request = RaftClientRequest;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        self.state.with_mut(|s| {
            if let Some(inbox) = s.inboxes.get_mut(&self.node_id) {
                if !inbox.is_empty() {
                    return Some(inbox.remove(0));
                }
            }
            None
        })
    }
}

impl<S> ExternalInterfaceOutgoing for InMemoryRaftExternalInterface<S>
where
    S: SharedState<InMemoryRaftExternalInterfaceState>,
{
    type Response = RaftClientResponse;

    fn send_response(&self, client_id: ClientId, response: Self::Response) {
        self.state.with_mut(|s| {
            s.outboxes
                .entry(self.node_id)
                .or_default()
                .push((client_id, response.clone()));
            s.responses_by_client
                .entry(client_id)
                .or_default()
                .push(response);
        });
    }
}
