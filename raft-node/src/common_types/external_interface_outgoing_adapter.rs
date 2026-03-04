// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::external_interface_outgoing::ExternalInterfaceOutgoing;

pub trait ExternalInterfaceOutgoingAdapter<Resp>:
    ExternalInterfaceOutgoing<Response = Resp>
{
}

impl<T, Resp> ExternalInterfaceOutgoingAdapter<Resp> for T where
    T: ExternalInterfaceOutgoing<Response = Resp>
{
}
