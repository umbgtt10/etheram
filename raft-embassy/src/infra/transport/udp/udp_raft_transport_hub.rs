// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::transport::udp::network_bus::MockNetDriver;
use crate::infra::transport::udp::network_bus::NetworkBus;
use crate::infra::transport::udp::network_config::get_node_config;
use crate::infra::transport::udp::network_config::make_node_resources;
use crate::infra::transport::udp::udp_raft_transport::inbox_endpoints;
use crate::infra::transport::udp::udp_raft_transport::outbox_endpoints;
use crate::infra::transport::udp::udp_raft_transport::run_udp_listener;
use crate::infra::transport::udp::udp_raft_transport::run_udp_sender;
use crate::infra::transport::udp::udp_raft_transport::RaftMessageReceiver;
use crate::infra::transport::udp::udp_raft_transport::RaftMessageSender;
use crate::infra::transport::udp::udp_raft_transport::UdpInboundRaftTransport;
use crate::infra::transport::udp::udp_raft_transport::UdpOutboundRaftTransport;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_net::Runner;
use embassy_net::Stack;

const NODE_COUNT: usize = 5;

static NETWORK_BUS: NetworkBus = NetworkBus::new();

pub struct UdpRaftTransportHub;

impl UdpRaftTransportHub {
    pub async fn initialize(
        spawner: &Spawner,
    ) -> Vec<(UdpInboundRaftTransport, UdpOutboundRaftTransport)> {
        let mut stacks: Vec<Stack<'static>> = Vec::with_capacity(NODE_COUNT);

        for node_index in 0..NODE_COUNT {
            let driver = MockNetDriver::new(node_index, &NETWORK_BUS);
            let config = get_node_config(node_index);
            let resources = make_node_resources();
            let seed = 0x0123_4567_89AB_CDEFu64 + node_index as u64;
            let (stack, runner) = embassy_net::new(driver, config, resources, seed);
            spawner
                .spawn(net_stack_task(node_index, runner))
                .expect("spawn net_stack_task");
            stacks.push(stack);
        }

        for stack in &stacks {
            stack.wait_link_up().await;
            stack.wait_config_up().await;
        }

        let mut transports = Vec::with_capacity(NODE_COUNT);

        for node_index in 0..NODE_COUNT {
            let (inbox_sender, inbox_receiver) = inbox_endpoints(node_index);
            let (outbox_sender, outbox_receiver) = outbox_endpoints(node_index);
            spawner
                .spawn(udp_listener_task(
                    node_index,
                    stacks[node_index],
                    inbox_sender,
                ))
                .expect("spawn udp_listener_task");
            spawner
                .spawn(udp_sender_task(
                    node_index,
                    stacks[node_index],
                    outbox_receiver,
                ))
                .expect("spawn udp_sender_task");
            transports.push((
                UdpInboundRaftTransport::new(inbox_receiver),
                UdpOutboundRaftTransport::new(outbox_sender),
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
async fn udp_listener_task(node_index: usize, stack: Stack<'static>, sender: RaftMessageSender) {
    run_udp_listener(node_index, stack, sender).await
}

#[embassy_executor::task(pool_size = 5)]
async fn udp_sender_task(node_index: usize, stack: Stack<'static>, receiver: RaftMessageReceiver) {
    run_udp_sender(node_index, stack, receiver).await
}
