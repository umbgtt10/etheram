// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::std_shared_state::StdSharedState;
use barechain_core::types::ClientId;
use barechain_core::types::PeerId;
use barechain_etheram_variants::implementations::eager_context_builder::EagerContextBuilder;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use barechain_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use barechain_etheram_variants::implementations::ibft::signature_scheme::BoxedSignatureScheme;
use barechain_etheram_variants::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use barechain_etheram_variants::implementations::in_memory_cache::InMemoryCache;
use barechain_etheram_variants::implementations::in_memory_external_interface::InMemoryExternalInterface;
use barechain_etheram_variants::implementations::in_memory_external_interface::InMemoryExternalInterfaceState;
use barechain_etheram_variants::implementations::in_memory_storage::InMemoryStorage;
use barechain_etheram_variants::implementations::in_memory_timer::InMemoryTimer;
use barechain_etheram_variants::implementations::in_memory_timer::InMemoryTimerState;
use barechain_etheram_variants::implementations::in_memory_transport::InMemoryTransport;
use barechain_etheram_variants::implementations::in_memory_transport::InMemoryTransportState;
use barechain_etheram_variants::implementations::no_op_observer::NoOpObserver;
use barechain_etheram_variants::implementations::shared_state::SharedState;
use barechain_etheram_variants::implementations::type_based_partitioner::TypeBasedPartitioner;
use barechain_etheram_variants::implementations::value_transfer_engine::ValueTransferEngine;
use etheram::common_types::account::Account;
use etheram::common_types::block::Block;
use etheram::common_types::types::Address;
use etheram::common_types::types::Balance;
use etheram::common_types::types::Hash;
use etheram::common_types::types::Height;
use etheram::etheram_node::EtheramNode;
use etheram::execution::execution_engine::BoxedExecutionEngine;
use etheram::execution::transaction_receipt::TransactionReceipt;
use etheram::executor::etheram_executor::EtheramExecutor;
use etheram::executor::outgoing::external_interface::client_response::ClientResponse;
use etheram::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram::incoming::external_interface::client_request::ClientRequest;
use etheram::incoming::incoming_sources::IncomingSources;
use etheram::incoming::timer::timer_event::TimerEvent;
use etheram::state::etheram_state::EtheramState;
use std::vec::Vec;

pub struct IbftCluster {
    validators: Vec<PeerId>,
    nodes: Vec<EtheramNode<IbftMessage>>,
    timer_state: StdSharedState<InMemoryTimerState>,
    transport_state: StdSharedState<InMemoryTransportState<IbftMessage>>,
    ei_state: StdSharedState<InMemoryExternalInterfaceState>,
}

impl IbftCluster {
    pub fn new(validators: Vec<PeerId>, genesis_accounts: Vec<(Address, Balance)>) -> Self {
        Self::new_with_validator_updates(validators, genesis_accounts, vec![])
    }

    pub fn new_with_validator_updates(
        validators: Vec<PeerId>,
        genesis_accounts: Vec<(Address, Balance)>,
        validator_updates: Vec<ValidatorSetUpdate>,
    ) -> Self {
        Self::new_with_validator_updates_and_signature_scheme_and_execution_engine(
            validators,
            genesis_accounts,
            validator_updates,
            |peer_id| Box::new(MockSignatureScheme::new(peer_id)),
            || Box::new(ValueTransferEngine),
        )
    }

    pub fn new_with_execution_engine_factory<F>(
        validators: Vec<PeerId>,
        genesis_accounts: Vec<(Address, Balance)>,
        execution_engine_factory: F,
    ) -> Self
    where
        F: Fn() -> BoxedExecutionEngine,
    {
        Self::new_with_validator_updates_and_signature_scheme_and_execution_engine(
            validators,
            genesis_accounts,
            vec![],
            |peer_id| Box::new(MockSignatureScheme::new(peer_id)),
            execution_engine_factory,
        )
    }

    pub fn new_with_validator_updates_and_signature_scheme<F>(
        validators: Vec<PeerId>,
        genesis_accounts: Vec<(Address, Balance)>,
        validator_updates: Vec<ValidatorSetUpdate>,
        signature_scheme_factory: F,
    ) -> Self
    where
        F: Fn(PeerId) -> BoxedSignatureScheme,
    {
        Self::new_with_validator_updates_and_signature_scheme_and_execution_engine(
            validators,
            genesis_accounts,
            validator_updates,
            signature_scheme_factory,
            || Box::new(ValueTransferEngine),
        )
    }

    pub fn new_with_validator_updates_and_signature_scheme_and_execution_engine<S, E>(
        validators: Vec<PeerId>,
        genesis_accounts: Vec<(Address, Balance)>,
        validator_updates: Vec<ValidatorSetUpdate>,
        signature_scheme_factory: S,
        execution_engine_factory: E,
    ) -> Self
    where
        S: Fn(PeerId) -> BoxedSignatureScheme,
        E: Fn() -> BoxedExecutionEngine,
    {
        let timer_state = StdSharedState::new(InMemoryTimerState::new());
        let transport_state = StdSharedState::new(InMemoryTransportState::<IbftMessage>::new());
        let ei_state = StdSharedState::new(InMemoryExternalInterfaceState::new());

        let mut nodes = Vec::new();
        for &peer_id in &validators {
            let mut storage = InMemoryStorage::new();
            for &(address, balance) in &genesis_accounts {
                storage = storage.with_genesis_account(address, balance);
            }

            let incoming = IncomingSources::new(
                Box::new(InMemoryTimer::new(peer_id, timer_state.clone())),
                Box::new(InMemoryExternalInterface::new(peer_id, ei_state.clone())),
                Box::new(InMemoryTransport::new(peer_id, transport_state.clone())),
            );
            let state = EtheramState::new(Box::new(storage), Box::new(InMemoryCache::new()));
            let outgoing = OutgoingSources::new(
                Box::new(InMemoryTimer::new(peer_id, timer_state.clone())),
                Box::new(InMemoryExternalInterface::new(peer_id, ei_state.clone())),
                Box::new(InMemoryTransport::new(peer_id, transport_state.clone())),
            );
            let executor = EtheramExecutor::new(outgoing);
            let protocol = IbftProtocol::new_with_validator_updates(
                validators.clone(),
                signature_scheme_factory(peer_id),
                validator_updates.clone(),
            )
            .with_execution_engine(execution_engine_factory());

            let node = EtheramNode::new(
                peer_id,
                incoming,
                state,
                executor,
                Box::new(EagerContextBuilder::new()),
                Box::new(protocol),
                Box::new(TypeBasedPartitioner::new()),
                execution_engine_factory(),
                Box::new(NoOpObserver),
            );
            nodes.push(node);
        }

        Self {
            validators,
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
        for i in 0..self.nodes.len() {
            while self.nodes[i].step() {}
        }
    }

    pub fn fire_timer(&self, node_index: usize, event: TimerEvent) {
        self.timer_state.with_mut(|state| {
            state.push_event(self.validators[node_index], event);
        });
    }

    pub fn inject_message(
        &self,
        receiver_index: usize,
        from_peer_id: PeerId,
        message: IbftMessage,
    ) {
        self.transport_state.with_mut(|state| {
            state.push_message(self.validators[receiver_index], from_peer_id, message);
        });
    }

    pub fn broadcast_message(&self, from_peer_id: PeerId, message: IbftMessage) {
        self.transport_state.with_mut(|state| {
            for &peer_id in &self.validators {
                state.push_message(peer_id, from_peer_id, message.clone());
            }
        });
    }

    pub fn node_height(&self, node_index: usize) -> Height {
        self.nodes[node_index].state().query_height()
    }

    pub fn node_stored_block(&self, node_index: usize, height: Height) -> Option<Block> {
        self.nodes[node_index].state().query_block(height)
    }

    pub fn node_account(&self, node_index: usize, address: Address) -> Option<Account> {
        self.nodes[node_index].state().query_account(address)
    }

    pub fn node_contract_storage(
        &self,
        node_index: usize,
        address: Address,
        slot: Hash,
    ) -> Option<Hash> {
        self.nodes[node_index]
            .state()
            .query_contract_storage(address, slot)
    }

    pub fn node_receipts(&self, node_index: usize, height: Height) -> Vec<TransactionReceipt> {
        self.nodes[node_index].state().query_receipts(height)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn submit_request(&self, node_index: usize, client_id: ClientId, request: ClientRequest) {
        self.ei_state.with_mut(|state| {
            state.push_request(self.validators[node_index], client_id, request);
        });
    }

    pub fn drain_client_responses(&self, client_id: ClientId) -> Vec<ClientResponse> {
        self.ei_state
            .with_mut(|state| state.drain_responses(client_id))
    }
}
