// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]

extern crate alloc;

pub mod channel_external_interface_macro;
pub mod channel_transport_hub;
pub mod client_facade_macro;
pub mod client_request_hub;
pub mod config;
pub mod embassy_shared_state;
pub mod heap;
pub mod logging;
pub mod network_bus;
pub mod network_config;
pub mod outbox_transport;
pub mod time_driver;
pub mod timer_channels;
