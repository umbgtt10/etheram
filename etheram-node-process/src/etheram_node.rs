// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::cache::cache_factory::build_cache;
use crate::infra::external_interface::external_interface_incoming_factory::build_external_interface_incoming;
use crate::infra::external_interface::external_interface_outgoing_factory::build_external_interface_outgoing;
use crate::infra::observer::observer_factory::build_observer;
use crate::infra::protocol::protocol_factory::build_protocol;
use crate::infra::scheduler::context_builder_factory::build_context_builder;
use crate::infra::scheduler::partitioner_factory::build_partitioner;
use crate::infra::storage::storage_factory::build_storage;
use crate::infra::timer::timer_input_factory::build_timer_input;
use crate::infra::timer::timer_output_factory::build_timer_output;
use crate::infra::transport::transport_incoming_factory::build_transport_incoming;
use crate::infra::transport::transport_outgoing_factory::build_transport_outgoing;
use etheram_core::types::PeerId;
use etheram_node::builders::etheram_node_builder::EtheramNodeBuilder;
use etheram_node::etheram_node::EtheramNode;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const IDLE_SLEEP_MS: u64 = 10;
const STATUS_INTERVAL_MS: u64 = 1000;

pub struct NodeRuntime {
    node: EtheramNode<()>,
}

impl NodeRuntime {
    pub fn new(peer_id: PeerId) -> Result<Self, String> {
        let node = EtheramNodeBuilder::<()>::new()
            .with_peer_id(peer_id)
            .with_timer_input(build_timer_input()?)
            .with_timer_output(build_timer_output()?)
            .with_transport_incoming(build_transport_incoming()?)
            .with_transport_outgoing(build_transport_outgoing()?)
            .with_external_interface_incoming(build_external_interface_incoming()?)
            .with_external_interface_outgoing(build_external_interface_outgoing()?)
            .with_storage(build_storage()?)
            .with_cache(build_cache()?)
            .with_protocol(build_protocol()?)
            .with_context_builder(build_context_builder()?)
            .with_partitioner(build_partitioner()?)
            .with_observer(build_observer()?)
            .build()
            .map_err(|error| format!("failed to build node runtime: {error:?}"))?;
        Ok(Self { node })
    }

    pub fn run_steps(&mut self, step_limit: u64) -> u64 {
        let mut executed = 0;
        while executed < step_limit {
            if !self.node.step() {
                break;
            }
            executed += 1;
        }
        executed
    }

    pub fn run_forever(&mut self) {
        let mut attempted_steps: u64 = 0;
        let mut progressed_steps: u64 = 0;
        let mut last_status_at = Instant::now();

        loop {
            let progressed = self.node.step();
            attempted_steps += 1;
            if progressed {
                progressed_steps += 1;
            } else {
                thread::sleep(Duration::from_millis(IDLE_SLEEP_MS));
            }

            if last_status_at.elapsed() >= Duration::from_millis(STATUS_INTERVAL_MS) {
                println!(
                    "node_status peer_id={} height={} last_hash={} attempted_steps={} progressed_steps={}",
                    self.node.peer_id(),
                    self.current_height(),
                    self.last_block_hash_short(),
                    attempted_steps,
                    progressed_steps
                );
                last_status_at = Instant::now();
            }
        }
    }

    fn current_height(&self) -> u64 {
        self.node.state().query_height()
    }

    fn last_block_hash_short(&self) -> String {
        let height = self.current_height();
        if height == 0 {
            return "none".to_string();
        }

        match self.node.state().query_block(height - 1) {
            Some(block) => {
                let hash = block.compute_hash();
                format!(
                    "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                    hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]
                )
            }
            None => "none".to_string(),
        }
    }
}
