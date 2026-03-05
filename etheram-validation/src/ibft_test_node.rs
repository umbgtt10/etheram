// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::node_common::shared_state::SharedState;
use etheram_core::node_common::spin_shared_state::SpinSharedState;
use etheram_core::types::ClientId;
use etheram_node::common_types::account::Account;
use etheram_node::common_types::types::{Address, Hash};
use etheram_node::etheram_node::EtheramNode;
use etheram_node::executor::etheram_executor::EtheramExecutor;
use etheram_node::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram_node::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram_node::implementations::eager_context_builder::EagerContextBuilder;
use etheram_node::implementations::ibft::ibft_message::IbftMessage;
use etheram_node::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_node::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use etheram_node::implementations::in_memory_cache::InMemoryCache;
use etheram_node::implementations::in_memory_external_interface::InMemoryExternalInterface;
use etheram_node::implementations::in_memory_external_interface::InMemoryExternalInterfaceState;
use etheram_node::implementations::in_memory_storage::InMemoryStorage;
use etheram_node::implementations::in_memory_timer::InMemoryTimer;
use etheram_node::implementations::in_memory_timer::InMemoryTimerState;
use etheram_node::implementations::in_memory_transport::InMemoryTransport;
use etheram_node::implementations::in_memory_transport::InMemoryTransportState;
use etheram_node::implementations::no_op_observer::NoOpObserver;
use etheram_node::implementations::type_based_partitioner::TypeBasedPartitioner;
use etheram_node::implementations::value_transfer_engine::ValueTransferEngine;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::incoming::incoming_sources::IncomingSources;
use etheram_node::incoming::timer::timer_event::TimerEvent;
use etheram_node::state::etheram_state::EtheramState;

pub const PEER_ID: u64 = 0;

pub struct IbftTestNode {
    node: EtheramNode<IbftMessage>,
    timer_state: SpinSharedState<InMemoryTimerState>,
    ei_state: SpinSharedState<InMemoryExternalInterfaceState>,
}

impl IbftTestNode {
    pub fn new(genesis_accounts: Vec<(Address, u128)>) -> Self {
        let timer_state = SpinSharedState::new(InMemoryTimerState::new());
        let transport_state = SpinSharedState::new(InMemoryTransportState::<IbftMessage>::new());
        let ei_state = SpinSharedState::new(InMemoryExternalInterfaceState::new());

        let mut storage = InMemoryStorage::new();
        for (address, balance) in genesis_accounts {
            storage = storage.with_genesis_account(address, balance);
        }

        let validators = vec![PEER_ID];
        let incoming = IncomingSources::new(
            Box::new(InMemoryTimer::new(PEER_ID, timer_state.clone())),
            Box::new(InMemoryExternalInterface::new(PEER_ID, ei_state.clone())),
            Box::new(InMemoryTransport::new(PEER_ID, transport_state.clone())),
        );
        let state = EtheramState::new(Box::new(storage), Box::new(InMemoryCache::new()));
        let outgoing = OutgoingSources::new(
            Box::new(InMemoryTimer::new(PEER_ID, timer_state.clone())),
            Box::new(InMemoryExternalInterface::new(PEER_ID, ei_state.clone())),
            Box::new(InMemoryTransport::new(PEER_ID, transport_state.clone())),
        );
        let executor = EtheramExecutor::new_with_peers(outgoing, validators.clone());

        let node = EtheramNode::new(
            PEER_ID,
            incoming,
            state,
            executor,
            Box::new(EagerContextBuilder::new()),
            Box::new(IbftProtocol::new(
                validators,
                Box::new(MockSignatureScheme::new(PEER_ID)),
            )),
            Box::new(TypeBasedPartitioner::new()),
            Box::new(ValueTransferEngine),
            Box::new(NoOpObserver),
        );

        Self {
            node,
            timer_state,
            ei_state,
        }
    }

    pub fn fire_timer(&self, event: TimerEvent) {
        self.timer_state.with_mut(|s| s.push_event(PEER_ID, event));
    }

    pub fn submit_request(&self, client_id: ClientId, request: ClientRequest) {
        self.ei_state
            .with_mut(|s| s.push_request(PEER_ID, client_id, request));
    }

    pub fn drain_responses(&self, client_id: ClientId) -> Vec<ClientResponse> {
        self.ei_state.with_mut(|s| s.drain_responses(client_id))
    }

    pub fn node_height(&self) -> u64 {
        self.node.state().query_height()
    }

    pub fn node_account(&self, address: Address) -> Option<Account> {
        self.node.state().query_account(address)
    }

    pub fn node_contract_storage(&self, address: Address, slot: Hash) -> Option<Hash> {
        self.node.state().query_contract_storage(address, slot)
    }

    pub fn snapshot_accounts_count(&self) -> usize {
        self.node.state().snapshot_accounts().len()
    }

    pub fn snapshot_contract_storage_count(&self) -> usize {
        self.node.state().snapshot_contract_storage().len()
    }

    pub fn step(&mut self) -> bool {
        self.node.step()
    }

    pub fn step_until_idle(&mut self) {
        while self.node.step() {}
    }
}
