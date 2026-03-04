// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use embassy_net::Config;
use embassy_net::Ipv4Address;
use embassy_net::Ipv4Cidr;
use embassy_net::StackResources;
use embassy_net::StaticConfigV4;

pub fn get_node_config(node_index: usize) -> Config {
    Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 0, 1, (node_index + 1) as u8), 24),
        gateway: Some(Ipv4Address::new(10, 0, 1, 254)),
        dns_servers: Default::default(),
    })
}

pub fn make_node_resources() -> &'static mut StackResources<4> {
    Box::leak(Box::new(StackResources::new()))
}
