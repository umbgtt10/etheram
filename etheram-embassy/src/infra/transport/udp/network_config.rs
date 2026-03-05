// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use embassy_net::Config;
use embassy_net::StackResources;

pub fn get_node_config(node_index: usize) -> Config {
    embassy_core::network_config::get_node_config(node_index, 0)
}

pub fn make_node_resources() -> &'static mut StackResources<4> {
    embassy_core::network_config::make_node_resources()
}
