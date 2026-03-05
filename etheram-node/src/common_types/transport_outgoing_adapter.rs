// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::transport_outgoing::TransportOutgoing;

pub trait TransportOutgoingAdapter<Msg>: TransportOutgoing<Message = Msg> {}

impl<T, Msg> TransportOutgoingAdapter<Msg> for T where T: TransportOutgoing<Message = Msg> {}
