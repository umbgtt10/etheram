// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use etheram_core::types::ClientId;

pub type GenericEiNotifyReceiver<'a, const NOTIFY_CAPACITY: usize> =
    Receiver<'a, CriticalSectionRawMutex, (), NOTIFY_CAPACITY>;

type RequestChannel<Req, const REQUEST_CAPACITY: usize> =
    Channel<CriticalSectionRawMutex, (ClientId, Req), REQUEST_CAPACITY>;

type ResponseChannel<Res, const RESPONSE_CAPACITY: usize> =
    Channel<CriticalSectionRawMutex, (ClientId, Res), RESPONSE_CAPACITY>;

type NotifyChannel<const NOTIFY_CAPACITY: usize> =
    Channel<CriticalSectionRawMutex, (), NOTIFY_CAPACITY>;

pub struct GenericClientRequestHub<
    Req,
    Res,
    const N: usize,
    const REQUEST_CAPACITY: usize,
    const RESPONSE_CAPACITY: usize,
    const NOTIFY_CAPACITY: usize,
> {
    requests: [RequestChannel<Req, REQUEST_CAPACITY>; N],
    responses: [ResponseChannel<Res, RESPONSE_CAPACITY>; N],
    notifications: [NotifyChannel<NOTIFY_CAPACITY>; N],
}

impl<
        Req,
        Res,
        const N: usize,
        const REQUEST_CAPACITY: usize,
        const RESPONSE_CAPACITY: usize,
        const NOTIFY_CAPACITY: usize,
    > GenericClientRequestHub<Req, Res, N, REQUEST_CAPACITY, RESPONSE_CAPACITY, NOTIFY_CAPACITY>
{
    pub const fn new() -> Self {
        Self {
            requests: [const { Channel::new() }; N],
            responses: [const { Channel::new() }; N],
            notifications: [const { Channel::new() }; N],
        }
    }

    pub fn try_receive_request(&self, node_index: usize) -> Option<(ClientId, Req)> {
        self.requests[node_index].try_receive().ok()
    }

    pub fn try_send_response(&self, node_index: usize, client_id: ClientId, response: Res) {
        let _ = self.responses[node_index].try_send((client_id, response));
    }

    pub fn send_request(&self, node_index: usize, client_id: ClientId, request: Req) {
        let _ = self.requests[node_index].try_send((client_id, request));
        let _ = self.notifications[node_index].try_send(());
    }

    pub async fn receive_response(&self, node_index: usize) -> (ClientId, Res) {
        self.responses[node_index].receive().await
    }

    pub fn notify_receiver(
        &self,
        node_index: usize,
    ) -> GenericEiNotifyReceiver<'_, NOTIFY_CAPACITY> {
        self.notifications[node_index].receiver()
    }
}

impl<
        Req,
        Res,
        const N: usize,
        const REQUEST_CAPACITY: usize,
        const RESPONSE_CAPACITY: usize,
        const NOTIFY_CAPACITY: usize,
    > Default
    for GenericClientRequestHub<Req, Res, N, REQUEST_CAPACITY, RESPONSE_CAPACITY, NOTIFY_CAPACITY>
{
    fn default() -> Self {
        Self::new()
    }
}
