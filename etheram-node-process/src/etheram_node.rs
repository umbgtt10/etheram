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
use crate::infra::std_shared_state::StdSharedState;
use crate::infra::storage::storage_factory::build_storage;
use crate::infra::sync::sync_handler::SyncHandler;
use crate::infra::sync::sync_sender::build_sync_sender;
use crate::infra::timer::timer_input_factory::build_timer_input;
use crate::infra::timer::timer_output_factory::build_timer_output;
use crate::infra::timer::timer_scheduler::TimerScheduler;
use crate::infra::transport::grpc_transport::grpc_transport_bus::GrpcTransportBus;
use crate::infra::transport::grpc_transport::sync_bus::SyncBus;
use crate::infra::transport::partitionable_transport::partition_control::spawn_partition_control_thread;
use crate::infra::transport::partitionable_transport::partition_table::PartitionTable;
use crate::infra::transport::partitionable_transport::shutdown_signal::is_shutdown_requested;
use crate::infra::transport::partitionable_transport::shutdown_signal::reset_shutdown;
use crate::infra::transport::transport_backend::TransportBackend;
use crate::infra::transport::transport_factory::build_transport_incoming;
use crate::infra::transport::transport_factory::build_transport_outgoing;
use etheram_core::types::PeerId;
use etheram_node::builders::execution_engine_builder::ExecutionEngineBuilder;
use etheram_node::etheram_node::EtheramNode;
use etheram_node::executor::etheram_executor::EtheramExecutor;
use etheram_node::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::in_memory_timer::InMemoryTimerState;
use etheram_node::incoming::incoming_sources::IncomingSources;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node::state::etheram_state::EtheramState;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const IDLE_SLEEP_MS: u64 = 10;
const STATUS_INTERVAL_MS: u64 = 1000;
const SYNC_RETRY_TICK_MS: u64 = 250;

pub struct NodeRuntime {
    node: EtheramNode<IbftMessage>,
    partition_table: Arc<PartitionTable>,
    sync_handler: SyncHandler,
    timer_scheduler: TimerScheduler,
}

impl NodeRuntime {
    pub fn new(
        peer_id: PeerId,
        listen_addr: &str,
        peer_addresses: &BTreeMap<PeerId, String>,
        validators: &[u64],
        db_path: &str,
    ) -> Result<Self, String> {
        let transport_backend = TransportBackend::from_env();
        let partition_table = Arc::new(PartitionTable::new());
        let blocked_count = partition_table.initialize_from_env()?;
        if blocked_count > 0 {
            println!(
                "partition_table initialized blocked_links={}",
                blocked_count
            );
        }

        let bus = Arc::new(GrpcTransportBus::new());
        let sync_bus = Arc::new(SyncBus::new());

        let timer_state = StdSharedState::new(InMemoryTimerState::new());
        let timer_input = build_timer_input(peer_id, timer_state.clone())?;
        let timer_output = build_timer_output(peer_id, timer_state.clone())?;
        timer_output.schedule(TimerEvent::ProposeBlock, 0);

        let transport_incoming = build_transport_incoming(
            &transport_backend,
            peer_id,
            listen_addr,
            Arc::clone(&bus),
            Arc::clone(&sync_bus),
        )?;
        let transport_outgoing = build_transport_outgoing(
            &transport_backend,
            peer_id,
            peer_addresses,
            Arc::clone(&partition_table),
            Arc::clone(&bus),
        )?;
        let sync_sender = build_sync_sender(
            &transport_backend,
            peer_id,
            peer_addresses,
            Arc::clone(&partition_table),
        );
        let external_interface_incoming = build_external_interface_incoming()?;
        let external_interface_outgoing = build_external_interface_outgoing()?;
        let storage = build_storage(db_path)?;
        let sync_storage = storage.clone();
        let cache = build_cache()?;
        let context_builder = build_context_builder()?;
        let protocol = build_protocol(validators)?;
        let partitioner = build_partitioner()?;
        let observer = build_observer()?;
        let execution_engine = ExecutionEngineBuilder::default()
            .build()
            .map_err(|error| format!("failed to build execution engine: {error:?}"))?;

        let incoming =
            IncomingSources::new(timer_input, external_interface_incoming, transport_incoming);
        let state = EtheramState::new(Box::new(storage), cache);
        let outgoing = OutgoingSources::new(
            timer_output,
            external_interface_outgoing,
            transport_outgoing,
        );
        let peers = peer_addresses.keys().copied().collect();
        let executor = EtheramExecutor::new_with_peers(outgoing, peers);
        let node = EtheramNode::new(
            peer_id,
            incoming,
            state,
            executor,
            context_builder,
            protocol,
            partitioner,
            execution_engine,
            observer,
        );

        let sync_handler = SyncHandler::new(sync_bus, sync_sender, Box::new(sync_storage));
        let timer_scheduler = TimerScheduler::new(timer_state);

        Ok(Self {
            node,
            partition_table,
            sync_handler,
            timer_scheduler,
        })
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
        reset_shutdown();
        if let Err(error) = spawn_partition_control_thread(Arc::clone(&self.partition_table)) {
            println!("partition_control_error {}", error);
        }

        let mut attempted_steps: u64 = 0;
        let mut progressed_steps: u64 = 0;
        let mut last_status_at = Instant::now();
        let mut last_sync_retry_tick_at = Instant::now();

        loop {
            if is_shutdown_requested() {
                println!("etheram-node-process shutdown requested");
                break;
            }

            self.timer_scheduler.tick(self.node.peer_id());

            let progressed = self.node.step();
            attempted_steps += 1;
            if progressed {
                progressed_steps += 1;
            } else {
                thread::sleep(Duration::from_millis(IDLE_SLEEP_MS));
            }

            self.sync_handler
                .process_sync_messages(self.node.peer_id(), self.node.state());

            if last_sync_retry_tick_at.elapsed() >= Duration::from_millis(SYNC_RETRY_TICK_MS) {
                self.sync_handler
                    .process_sync_timeouts(self.node.peer_id(), self.current_height());
                last_sync_retry_tick_at = Instant::now();
            }

            if last_status_at.elapsed() >= Duration::from_millis(STATUS_INTERVAL_MS) {
                self.sync_handler
                    .broadcast_status(self.current_height(), self.last_block_hash());
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

    fn last_block_hash(&self) -> [u8; 32] {
        let height = self.current_height();
        if height == 0 {
            return [0u8; 32];
        }

        self.node
            .state()
            .query_block(height - 1)
            .map(|block| block.compute_hash())
            .unwrap_or([0u8; 32])
    }

    fn last_block_hash_short(&self) -> String {
        let hash = self.last_block_hash();
        if hash == [0u8; 32] {
            return "none".to_string();
        }

        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]
        )
    }
}
