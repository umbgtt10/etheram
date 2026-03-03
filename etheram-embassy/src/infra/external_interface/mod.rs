// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "channel-external-interface")]
pub mod channel;
pub mod client_facade;
#[cfg(feature = "udp-external-interface")]
pub mod udp;
