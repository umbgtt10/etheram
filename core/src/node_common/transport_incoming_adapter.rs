// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::transport_incoming::TransportIncoming;

pub trait TransportIncomingAdapter<Msg>: TransportIncoming<Message = Msg> {}

impl<T, Msg> TransportIncomingAdapter<Msg> for T where T: TransportIncoming<Message = Msg> {}
