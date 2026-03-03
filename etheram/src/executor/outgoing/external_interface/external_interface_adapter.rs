// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use alloc::boxed::Box;
use barechain_core::external_interface_outgoing::ExternalInterfaceOutgoing;
use barechain_core::types::ClientId;

impl<Resp> ExternalInterfaceOutgoing for Box<dyn ExternalInterfaceOutgoingAdapter<Resp>>
where
    Resp: 'static,
{
    type Response = Resp;

    fn send_response(&self, client_id: ClientId, response: Self::Response) {
        (**self).send_response(client_id, response)
    }
}
