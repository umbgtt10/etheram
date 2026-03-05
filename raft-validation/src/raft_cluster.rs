// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::std_shared_state::StdSharedState;
use etheram_core::node_common::external_interface_incoming_adapter::ExternalInterfaceIncomingAdapter;
use etheram_core::node_common::external_interface_outgoing_adapter::ExternalInterfaceOutgoingAdapter;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::node_common::timer_input_adapter::TimerInputAdapter;
use etheram_core::node_common::timer_output_adapter::TimerOutputAdapter;
use etheram_core::node_common::transport_incoming_adapter::TransportIncomingAdapter;
use etheram_core::node_common::transport_outgoing_adapter::TransportOutgoingAdapter;
use etheram_core::types::ClientId;
use etheram_core::types::PeerId;
use raft_node::brain::protocol::boxed_protocol::BoxedRaftProtocol;
use raft_node::brain::protocol::message::RaftMessage;
use raft_node::common_types::cache_adapter::CacheAdapter;
use raft_node::common_types::node_role::NodeRole;
use raft_node::common_types::state_machine::RaftStateMachine;
use raft_node::common_types::storage_adapter::StorageAdapter;
use raft_node::context::context_builder::RaftContextBuilder;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::executor::outgoing::outgoing_sources::RaftOutgoingSources;
use raft_node::executor::raft_executor::RaftExecutor;
use raft_node::implementations::eager_raft_context_builder::EagerRaftContextBuilder;
use raft_node::implementations::in_memory_raft_cache::InMemoryRaftCache;
use raft_node::implementations::in_memory_raft_external_interface::InMemoryRaftExternalInterface;
use raft_node::implementations::in_memory_raft_external_interface::InMemoryRaftExternalInterfaceState;
use raft_node::implementations::in_memory_raft_state_machine::InMemoryRaftStateMachine;
use raft_node::implementations::in_memory_raft_storage::InMemoryRaftStorage;
use raft_node::implementations::in_memory_raft_timer::InMemoryRaftTimer;
use raft_node::implementations::in_memory_raft_timer::InMemoryRaftTimerState;
use raft_node::implementations::in_memory_raft_transport::InMemoryRaftTransport;
use raft_node::implementations::in_memory_raft_transport::InMemoryRaftTransportState;
use raft_node::implementations::no_op_raft_observer::NoOpRaftObserver;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::implementations::type_based_raft_partitioner::TypeBasedRaftPartitioner;
use raft_node::incoming::external_interface::client_request::RaftClientRequest;
use raft_node::incoming::incoming_sources::RaftIncomingSources;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_node::observer::RaftObserver;
use raft_node::partitioner::partition::RaftPartitioner;
use raft_node::raft_node::RaftNode;
use raft_node::state::raft_state::RaftState;
use std::vec::Vec;

pub struct RaftCluster {
    peer_ids: Vec<PeerId>,
    nodes: Vec<RaftNode<Vec<u8>>>,
    timer_state: StdSharedState<InMemoryRaftTimerState>,
    transport_state: StdSharedState<InMemoryRaftTransportState<Vec<u8>>>,
    ei_state: StdSharedState<InMemoryRaftExternalInterfaceState>,
}

impl RaftCluster {
    pub fn new(node_count: usize) -> Self {
        let peer_ids: Vec<PeerId> = (1..=(node_count as u64)).collect();
        let timer_state = StdSharedState::new(InMemoryRaftTimerState::new());
        let transport_state = StdSharedState::new(InMemoryRaftTransportState::<Vec<u8>>::new());
        let ei_state = StdSharedState::new(InMemoryRaftExternalInterfaceState::new());

        let mut nodes = Vec::new();
        for &peer_id in &peer_ids {
            let other_peers: Vec<PeerId> = peer_ids
                .iter()
                .filter(|&&p| p != peer_id)
                .copied()
                .collect();

            let timer_in: Box<dyn TimerInputAdapter<RaftTimerEvent>> =
                Box::new(InMemoryRaftTimer::new(peer_id, timer_state.clone()));
            let timer_out: Box<dyn TimerOutputAdapter<RaftTimerEvent, u64>> =
                Box::new(InMemoryRaftTimer::new(peer_id, timer_state.clone()));
            let transport_in: Box<dyn TransportIncomingAdapter<RaftMessage<Vec<u8>>>> =
                Box::new(InMemoryRaftTransport::new(peer_id, transport_state.clone()));
            let transport_out: Box<dyn TransportOutgoingAdapter<RaftMessage<Vec<u8>>>> =
                Box::new(InMemoryRaftTransport::new(peer_id, transport_state.clone()));
            let ei_in: Box<dyn ExternalInterfaceIncomingAdapter<RaftClientRequest>> = Box::new(
                InMemoryRaftExternalInterface::new(peer_id, ei_state.clone()),
            );
            let ei_out: Box<dyn ExternalInterfaceOutgoingAdapter<RaftClientResponse>> = Box::new(
                InMemoryRaftExternalInterface::new(peer_id, ei_state.clone()),
            );

            let storage: Box<dyn StorageAdapter<Vec<u8>, Key = (), Value = ()>> =
                Box::new(InMemoryRaftStorage::new());
            let cache: Box<dyn CacheAdapter<Key = (), Value = ()>> =
                Box::new(InMemoryRaftCache::new());
            let context_builder: Box<dyn RaftContextBuilder<Vec<u8>>> =
                Box::new(EagerRaftContextBuilder::new(peer_id, other_peers.clone()));
            let brain: BoxedRaftProtocol<Vec<u8>> = Box::new(RaftProtocol::<Vec<u8>>::new());
            let partitioner: Box<dyn RaftPartitioner<Vec<u8>>> =
                Box::new(TypeBasedRaftPartitioner::new());
            let state_machine: Box<dyn RaftStateMachine> =
                Box::new(InMemoryRaftStateMachine::new());
            let observer: Box<dyn RaftObserver> = Box::new(NoOpRaftObserver::new());

            let incoming = RaftIncomingSources::new(timer_in, ei_in, transport_in);
            let outgoing = RaftOutgoingSources::new(timer_out, ei_out, transport_out);
            let executor = RaftExecutor::new_with_peers(outgoing, other_peers);
            let state = RaftState::new(storage, cache);

            let node = RaftNode::new(
                peer_id,
                incoming,
                state,
                executor,
                context_builder,
                brain,
                partitioner,
                state_machine,
                observer,
            );
            nodes.push(node);
        }

        Self {
            peer_ids,
            nodes,
            timer_state,
            transport_state,
            ei_state,
        }
    }

    pub fn step(&mut self, node_index: usize) -> bool {
        self.nodes[node_index].step()
    }

    pub fn drain(&mut self, node_index: usize) {
        while self.nodes[node_index].step() {}
    }

    pub fn drain_all(&mut self) {
        loop {
            let mut any = false;
            for i in 0..self.nodes.len() {
                if self.nodes[i].step() {
                    any = true;
                }
            }
            if !any {
                break;
            }
        }
    }

    pub fn drain_except(&mut self, excluded: &[usize]) {
        loop {
            let mut any = false;
            for i in 0..self.nodes.len() {
                if !excluded.contains(&i) && self.nodes[i].step() {
                    any = true;
                }
            }
            if !any {
                break;
            }
        }
    }

    pub fn fire_timer(&self, node_index: usize, event: RaftTimerEvent) {
        self.timer_state.with_mut(|state| {
            state.push_event(self.peer_ids[node_index], event);
        });
    }

    pub fn fire_timer_all(&self, event: RaftTimerEvent) {
        for i in 0..self.peer_ids.len() {
            self.fire_timer(i, event);
        }
    }

    pub fn inject_message(
        &self,
        receiver_index: usize,
        from_peer_id: PeerId,
        message: RaftMessage<Vec<u8>>,
    ) {
        self.transport_state.with_mut(|state| {
            state.push_message(self.peer_ids[receiver_index], from_peer_id, message);
        });
    }

    pub fn submit_command(&self, node_index: usize, client_id: ClientId, payload: Vec<u8>) {
        self.ei_state.with_mut(|state| {
            state.push_request(
                self.peer_ids[node_index],
                client_id,
                RaftClientRequest::Command(payload),
            );
        });
    }

    pub fn submit_query(&self, node_index: usize, client_id: ClientId, key: &str) {
        self.ei_state.with_mut(|state| {
            state.push_request(
                self.peer_ids[node_index],
                client_id,
                RaftClientRequest::Query(key.into()),
            );
        });
    }

    pub fn drain_responses(&self, node_index: usize) -> Vec<(ClientId, RaftClientResponse)> {
        self.ei_state
            .with_mut(|state| state.drain_responses(self.peer_ids[node_index]))
    }

    pub fn drain_client_responses(&self, client_id: ClientId) -> Vec<RaftClientResponse> {
        self.ei_state
            .with_mut(|state| state.drain_client_responses(client_id))
    }

    pub fn node_role(&self, node_index: usize) -> NodeRole {
        self.nodes[node_index].state().query_role()
    }

    pub fn node_commit_index(&self, node_index: usize) -> u64 {
        self.nodes[node_index].state().query_commit_index()
    }

    pub fn node_last_applied(&self, node_index: usize) -> u64 {
        self.nodes[node_index].state().query_last_applied()
    }

    pub fn node_log_length(&self, node_index: usize) -> u64 {
        self.nodes[node_index].state().query_log_length()
    }

    pub fn node_current_term(&self, node_index: usize) -> u64 {
        self.nodes[node_index].state().query_current_term()
    }

    pub fn node_leader_id(&self, node_index: usize) -> Option<PeerId> {
        self.nodes[node_index].state().query_leader_id()
    }

    pub fn node_peer_id(&self, node_index: usize) -> PeerId {
        self.peer_ids[node_index]
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn find_leader(&self) -> Option<usize> {
        self.nodes
            .iter()
            .position(|n| n.state().query_role() == NodeRole::Leader)
    }

    pub fn elect_leader(&mut self) -> usize {
        self.fire_timer(0, RaftTimerEvent::ElectionTimeout);
        self.drain_all();
        self.find_leader()
            .expect("no leader elected after election timeout")
    }
}
