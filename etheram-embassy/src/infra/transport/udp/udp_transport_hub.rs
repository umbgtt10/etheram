// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::udp::network_config::get_node_config;
use crate::infra::transport::udp::network_config::make_node_resources;
use crate::infra::transport::udp::udp_transport::inbox_endpoints;
use crate::infra::transport::udp::udp_transport::outbox_endpoints;
use crate::infra::transport::udp::udp_transport::run_udp_listener;
use crate::infra::transport::udp::udp_transport::run_udp_sender;
use crate::infra::transport::udp::udp_transport::IbftMessageReceiver;
use crate::infra::transport::udp::udp_transport::IbftMessageSender;
use crate::infra::transport::udp::udp_transport::UdpInboundTransport;
use crate::infra::transport::udp::udp_transport::UdpOutboundTransport;
use alloc::vec::Vec;
use embassy_core::network_bus::MockNetDriver;
use embassy_core::network_bus::NetworkBus;
use embassy_executor::Spawner;
use embassy_net::Runner;
use embassy_net::Stack;

static NETWORK_BUS: NetworkBus = NetworkBus::new();

pub struct UdpTransportHub;

impl UdpTransportHub {
    pub async fn initialize(spawner: &Spawner) -> Vec<(UdpInboundTransport, UdpOutboundTransport)> {
        let mut stacks: Vec<Stack<'static>> = Vec::with_capacity(5);

        for node_index in 0..5 {
            let driver = MockNetDriver::new(node_index, &NETWORK_BUS);
            let config = get_node_config(node_index);
            let resources = make_node_resources();
            let seed = 0x0123_4567_89AB_CDEFu64 + node_index as u64;

            let (stack, runner) = embassy_net::new(driver, config, resources, seed);

            spawner.spawn(net_stack_task(node_index, runner)).unwrap();
            stacks.push(stack);
        }

        for stack in &stacks {
            stack.wait_link_up().await;
            stack.wait_config_up().await;
        }

        let mut transports = Vec::with_capacity(5);

        for node_index in 0..5 {
            let (inbox_sender, inbox_receiver) = inbox_endpoints(node_index);
            let (outbox_sender, outbox_receiver) = outbox_endpoints(node_index);

            spawner
                .spawn(udp_listener_task(
                    node_index,
                    stacks[node_index],
                    inbox_sender,
                ))
                .unwrap();

            spawner
                .spawn(udp_sender_task(
                    node_index,
                    stacks[node_index],
                    outbox_receiver,
                ))
                .unwrap();

            transports.push((
                UdpInboundTransport::new(inbox_receiver),
                UdpOutboundTransport::new(outbox_sender),
            ));
        }

        transports
    }
}

#[embassy_executor::task(pool_size = 5)]
async fn net_stack_task(node_index: usize, mut runner: Runner<'static, MockNetDriver>) {
    let _ = node_index;
    runner.run().await
}

#[embassy_executor::task(pool_size = 5)]
async fn udp_listener_task(node_index: usize, stack: Stack<'static>, sender: IbftMessageSender) {
    run_udp_listener(node_index, stack, sender).await
}

#[embassy_executor::task(pool_size = 5)]
async fn udp_sender_task(node_index: usize, stack: Stack<'static>, receiver: IbftMessageReceiver) {
    run_udp_sender(node_index, stack, receiver).await
}
