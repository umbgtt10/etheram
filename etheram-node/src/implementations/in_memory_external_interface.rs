// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::executor::outgoing::external_interface::client_response::ClientResponse;
use crate::implementations::shared_state::SharedState;
use crate::incoming::external_interface::client_request::ClientRequest;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use etheram_core::external_interface_incoming::ExternalInterfaceIncoming;
use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use etheram_core::types::ClientId;

pub struct InMemoryExternalInterfaceState {
    requests: BTreeMap<u64, Vec<(ClientId, ClientRequest)>>,
    responses: BTreeMap<ClientId, Vec<ClientResponse>>,
}

impl InMemoryExternalInterfaceState {
    pub fn new() -> Self {
        Self {
            requests: BTreeMap::new(),
            responses: BTreeMap::new(),
        }
    }

    pub fn push_request(&mut self, node_id: u64, client_id: ClientId, request: ClientRequest) {
        self.requests
            .entry(node_id)
            .or_default()
            .push((client_id, request));
    }

    pub fn drain_responses(&mut self, client_id: ClientId) -> Vec<ClientResponse> {
        self.responses.remove(&client_id).unwrap_or_default()
    }
}

impl Default for InMemoryExternalInterfaceState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct InMemoryExternalInterface<S>
where
    S: SharedState<InMemoryExternalInterfaceState>,
{
    node_id: u64,
    state: S,
}

impl<S> InMemoryExternalInterface<S>
where
    S: SharedState<InMemoryExternalInterfaceState>,
{
    pub fn new(node_id: u64, state: S) -> Self {
        state.with_mut(|state| {
            state.requests.insert(node_id, Vec::new());
        });
        Self { node_id, state }
    }
}

impl<S> ExternalInterfaceIncoming for InMemoryExternalInterface<S>
where
    S: SharedState<InMemoryExternalInterfaceState>,
{
    type Request = ClientRequest;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        self.state.with_mut(|state| {
            if let Some(queue) = state.requests.get_mut(&self.node_id) {
                if !queue.is_empty() {
                    return Some(queue.remove(0));
                }
            }
            None
        })
    }
}

impl<S> ExternalInterfaceOutgoing for InMemoryExternalInterface<S>
where
    S: SharedState<InMemoryExternalInterfaceState>,
{
    type Response = ClientResponse;

    fn send_response(&self, client_id: ClientId, response: Self::Response) {
        self.state.with_mut(|state| {
            state.responses.entry(client_id).or_default().push(response);
        });
    }
}
