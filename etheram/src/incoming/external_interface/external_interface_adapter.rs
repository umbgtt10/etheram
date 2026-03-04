// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common_types::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use alloc::boxed::Box;
use etheram_core::{external_interface_incoming::ExternalInterfaceIncoming, types::ClientId};

impl<Req> ExternalInterfaceIncoming for Box<dyn ExternalInterfaceIncomingAdapter<Req>>
where
    Req: 'static,
{
    type Request = Req;

    fn poll_request(&self) -> Option<(ClientId, Self::Request)> {
        (**self).poll_request()
    }
}
