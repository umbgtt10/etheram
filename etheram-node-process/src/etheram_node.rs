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
use crate::infra::storage::in_memory_storage::InMemoryStorage;
use crate::infra::storage::storage_factory::build_storage;
use crate::infra::sync::sync_import::decode_and_validate_blocks;
use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::sync::sync_sender::build_sync_sender;
use crate::infra::sync::sync_sender::SyncSender;
use crate::infra::sync::sync_state::SyncState;
use crate::infra::timer::timer_input_factory::build_timer_input;
use crate::infra::timer::timer_output_factory::build_timer_output;
use crate::infra::transport::grpc_transport::sync_bus::dequeue_sync_for;
use crate::infra::transport::grpc_transport::wire_ibft_message::serialize_block;
use crate::infra::transport::partitionable_transport::partition_control::spawn_partition_control_thread;
use crate::infra::transport::partitionable_transport::partition_table::global_partition_table;
use crate::infra::transport::partitionable_transport::shutdown_signal::is_shutdown_requested;
use crate::infra::transport::partitionable_transport::shutdown_signal::reset_shutdown;
use crate::infra::transport::transport_backend::TransportBackend;
use crate::infra::transport::transport_factory::build_transport_incoming;
use crate::infra::transport::transport_factory::build_transport_outgoing;
use etheram_core::node_common::shared_state::SharedState;
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
use std::thread;
use std::time::Duration;
use std::time::Instant;

const IDLE_SLEEP_MS: u64 = 10;
const PROPOSE_TICK_MS: u64 = 250;
const STATUS_INTERVAL_MS: u64 = 1000;
const TIMEOUT_TICK_MS: u64 = 1500;
const SYNC_MAX_BLOCKS_PER_REQUEST: u64 = 64;

pub struct NodeRuntime {
    node: EtheramNode<IbftMessage>,
    sync_sender: Box<dyn SyncSender>,
    sync_storage: InMemoryStorage,
    sync_state: SyncState,
    timer_state: StdSharedState<InMemoryTimerState>,
}

impl NodeRuntime {
    pub fn new(
        peer_id: PeerId,
        listen_addr: &str,
        peer_addresses: &BTreeMap<PeerId, String>,
        validators: &[u64],
    ) -> Result<Self, String> {
        let transport_backend = TransportBackend::from_env();
        let blocked_count = global_partition_table().initialize_from_env()?;
        if blocked_count > 0 {
            println!(
                "partition_table initialized blocked_links={}",
                blocked_count
            );
        }

        let timer_state = StdSharedState::new(InMemoryTimerState::new());
        let timer_input = build_timer_input(peer_id, timer_state.clone())?;
        let timer_output = build_timer_output(peer_id, timer_state.clone())?;
        timer_output.schedule(TimerEvent::ProposeBlock, 0);

        let transport_incoming =
            build_transport_incoming(&transport_backend, peer_id, listen_addr)?;
        let transport_outgoing =
            build_transport_outgoing(&transport_backend, peer_id, peer_addresses)?;
        let sync_sender = build_sync_sender(&transport_backend, peer_id, peer_addresses);
        let external_interface_incoming = build_external_interface_incoming()?;
        let external_interface_outgoing = build_external_interface_outgoing()?;
        let storage = build_storage()?;
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
        Ok(Self {
            node,
            sync_sender,
            sync_storage,
            sync_state: SyncState::new(),
            timer_state,
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
        if let Err(error) = spawn_partition_control_thread() {
            println!("partition_control_error {}", error);
        }

        let mut attempted_steps: u64 = 0;
        let mut progressed_steps: u64 = 0;
        let mut last_propose_tick_at = Instant::now();
        let mut last_status_at = Instant::now();
        let mut last_timeout_tick_at = Instant::now();

        loop {
            if is_shutdown_requested() {
                println!("etheram-node-process shutdown requested");
                break;
            }

            if last_propose_tick_at.elapsed() >= Duration::from_millis(PROPOSE_TICK_MS) {
                let node_id = self.node.peer_id();
                self.timer_state
                    .with_mut(|state| state.push_event(node_id, TimerEvent::ProposeBlock));
                last_propose_tick_at = Instant::now();
            }

            if last_timeout_tick_at.elapsed() >= Duration::from_millis(TIMEOUT_TICK_MS) {
                let node_id = self.node.peer_id();
                self.timer_state
                    .with_mut(|state| state.push_event(node_id, TimerEvent::TimeoutRound));
                last_timeout_tick_at = Instant::now();
            }

            let progressed = self.node.step();
            attempted_steps += 1;
            if progressed {
                progressed_steps += 1;
            } else {
                thread::sleep(Duration::from_millis(IDLE_SLEEP_MS));
            }

            self.process_sync_messages();

            if last_status_at.elapsed() >= Duration::from_millis(STATUS_INTERVAL_MS) {
                self.sync_sender
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

    fn process_sync_messages(&mut self) {
        while let Some((peer_id, message)) = dequeue_sync_for(self.node.peer_id()) {
            match message {
                SyncMessage::Status { height, .. } => {
                    self.sync_state.observe_status(peer_id, height);
                    let local_height = self.current_height();
                    if let Some(lag_distance) = self.sync_state.lag_distance(local_height) {
                        println!(
                            "sync_state peer_id={} lagging_by={} best_peer_height={}",
                            self.node.peer_id(),
                            lag_distance,
                            local_height + lag_distance
                        );
                    }

                    if let Some((target_peer, from_height, max_blocks)) = self
                        .sync_state
                        .next_request(local_height, SYNC_MAX_BLOCKS_PER_REQUEST)
                    {
                        self.sync_sender.send_to_peer(
                            target_peer,
                            &SyncMessage::GetBlocks {
                                from_height,
                                max_blocks,
                            },
                        );
                    }
                }
                SyncMessage::GetBlocks {
                    from_height,
                    max_blocks,
                } => {
                    self.handle_get_blocks(peer_id, from_height, max_blocks);
                }
                SyncMessage::Blocks {
                    start_height,
                    block_payloads,
                } => {
                    self.handle_blocks(peer_id, start_height, &block_payloads);
                }
            }
        }
    }

    fn handle_get_blocks(&mut self, peer_id: PeerId, from_height: u64, max_blocks: u64) {
        let mut block_payloads = Vec::new();
        for offset in 0..max_blocks {
            let height = from_height + offset;
            let Some(block) = self.node.state().query_block(height) else {
                break;
            };
            let Ok(payload) = serialize_block(&block) else {
                break;
            };
            block_payloads.push(payload);
        }

        self.sync_sender.send_to_peer(
            peer_id,
            &SyncMessage::Blocks {
                start_height: from_height,
                block_payloads,
            },
        );
    }

    fn handle_blocks(&mut self, peer_id: PeerId, start_height: u64, block_payloads: &[Vec<u8>]) {
        let local_height = self.current_height();
        let decoded_blocks = decode_and_validate_blocks(local_height, start_height, block_payloads);
        let decoded = decoded_blocks
            .as_ref()
            .map(|blocks| blocks.len() as u64)
            .unwrap_or(0);

        let malformed = decoded_blocks.is_none();
        let empty_while_lagging =
            block_payloads.is_empty() && self.sync_state.lag_distance(local_height).is_some();

        if let Some(blocks) = decoded_blocks.as_ref() {
            if !empty_while_lagging {
                self.sync_storage.apply_synced_blocks(blocks);
            }
        }

        let completed = if malformed || empty_while_lagging {
            false
        } else {
            self.sync_state
                .complete_in_flight_request(peer_id, start_height)
        };

        let failed = if malformed || empty_while_lagging {
            self.sync_state
                .fail_in_flight_request(peer_id, start_height)
        } else {
            false
        };

        if completed || failed {
            if let Some((target_peer, from_height, max_blocks)) = self
                .sync_state
                .next_request(self.current_height(), SYNC_MAX_BLOCKS_PER_REQUEST)
            {
                self.sync_sender.send_to_peer(
                    target_peer,
                    &SyncMessage::GetBlocks {
                        from_height,
                        max_blocks,
                    },
                );
            }
        }
        println!(
            "sync_blocks peer_id={} from_peer={} start_height={} payloads={} decoded={} malformed={} empty_while_lagging={} completed_request={} failed_request={}",
            self.node.peer_id(),
            peer_id,
            start_height,
            block_payloads.len(),
            decoded,
            malformed,
            empty_while_lagging,
            completed,
            failed
        );
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
