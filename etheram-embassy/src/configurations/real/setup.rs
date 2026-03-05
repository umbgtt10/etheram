// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::infrastructure::RealInfrastructure;
use super::node_infra_slot::NodeInfraSlot;
use crate::cancellation_token::CancellationToken;
use crate::config::MAX_NODES;
use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::external_interface::udp::udp_external_interface::udp_ei_notify_receiver;
use crate::infra::external_interface::udp::udp_external_interface::UdpExternalInterface;
use crate::infra::storage::captured_wal_writer::CapturedWalWriter;
use crate::infra::storage::semihosting::semihosting_wal_writer::SemihostingWalWriter;
use crate::infra::timer::channel::timer_channels::TIMER_CHANNELS;
use crate::infra::transport::udp::udp_transport::IbftMessageReceiver;
use crate::infra::transport::udp::udp_transport_hub::UdpTransportHub;
use crate::semihosting_observer::SemihostingObserver;
use crate::spawned_node::SpawnedNode;
use crate::spawned_node::TimerReceiver;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_futures::select::select4;
use embassy_futures::select::Either4;
use etheram_core::types::PeerId;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Hash;
use etheram_node::common_types::types::Height;
use etheram_node::etheram_node::EtheramNode;
use etheram_node::execution::execution_engine::BoxedExecutionEngine;
use etheram_node::executor::etheram_executor::EtheramExecutor;
use etheram_node::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram_node::incoming::incoming_sources::IncomingSources;
use etheram_node::observer::EventLevel;
use etheram_node::state::etheram_state::EtheramState;
use etheram_variants::implementations::eager_context_builder::EagerContextBuilder;
use etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_variants::implementations::ibft::ed25519_signature_scheme::Ed25519SignatureScheme;
use etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use etheram_variants::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use etheram_variants::implementations::ibft::wal_writer::CompositeWalWriter;
use etheram_variants::implementations::in_memory_cache::InMemoryCache;
use etheram_variants::implementations::in_memory_timer::InMemoryTimer;
use etheram_variants::implementations::in_memory_timer::InMemoryTimerState;
use etheram_variants::implementations::in_memory_transport::InMemoryTransport;
use etheram_variants::implementations::in_memory_transport::InMemoryTransportState;
use etheram_variants::implementations::shared_state::SharedState;
use etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use etheram_variants::implementations::type_based_partitioner::TypeBasedPartitioner;

fn create_execution_engine() -> BoxedExecutionEngine {
    Box::new(TinyEvmEngine)
}

pub async fn init(spawner: &Spawner) -> RealInfrastructure {
    let sender: Address = [1u8; 20];
    let receiver: Address = [2u8; 20];
    let act10_sender: Address = [3u8; 20];
    let mut transports = UdpTransportHub::initialize(spawner).await;
    let slots = core::array::from_fn(|i| {
        let (inbound, outbound) = transports.remove(0);
        Some(
            NodeInfraSlot::new(i as PeerId, inbound, outbound)
                .with_genesis_account(sender, 1_000)
                .with_genesis_account(receiver, 200)
                .with_genesis_account(act10_sender, 1_000),
        )
    });
    RealInfrastructure::new(slots)
}

impl SpawnedNode {
    pub fn new(
        spawner: &Spawner,
        node_index: usize,
        slot: NodeInfraSlot,
        cancel: &'static CancellationToken,
    ) -> Self {
        let peer_id = node_index as PeerId;
        let validators: Vec<PeerId> = (0..MAX_NODES as u64).collect();
        let peers: Vec<PeerId> = validators
            .iter()
            .filter(|&&p| p != peer_id)
            .copied()
            .collect();

        let incoming = IncomingSources::new(
            Box::new(InMemoryTimer::new(peer_id, slot.timer_state.clone())),
            Box::new(UdpExternalInterface::new(node_index)),
            Box::new(InMemoryTransport::new(
                peer_id,
                slot.transport_state.clone(),
            )),
        );

        let state = EtheramState::new(Box::new(slot.storage), Box::new(InMemoryCache::new()));

        let outgoing = OutgoingSources::new(
            Box::new(InMemoryTimer::new(peer_id, slot.timer_state.clone())),
            Box::new(UdpExternalInterface::new(node_index)),
            Box::new(slot.outbound),
        );

        let executor = EtheramExecutor::new_with_peers(outgoing, peers);

        let wal_state: EmbassySharedState<Option<ConsensusWal>> = EmbassySharedState::new(None);
        let cert_state: EmbassySharedState<Option<PreparedCertificate>> =
            EmbassySharedState::new(None);
        let capture = Box::new(CapturedWalWriter::new(
            wal_state.clone(),
            cert_state.clone(),
        ));
        let persister = Box::new(SemihostingWalWriter::new(peer_id));
        let wal_writer = Box::new(CompositeWalWriter::new(persister, capture));

        let ibft_proto = IbftProtocol::new_with_validator_updates(
            validators,
            Box::new(Ed25519SignatureScheme::new(peer_id)),
            alloc::vec![ValidatorSetUpdate::new(5, (0..MAX_NODES as u64).collect())],
        )
        .with_execution_engine(create_execution_engine())
        .with_wal_writer(wal_writer);

        let node = EtheramNode::new(
            peer_id,
            incoming,
            state,
            executor,
            Box::new(EagerContextBuilder::new()),
            Box::new(ibft_proto),
            Box::new(TypeBasedPartitioner::new()),
            create_execution_engine(),
            Box::new(SemihostingObserver::new(EventLevel::Essential)),
        );

        let height = EmbassySharedState::new(0u64);
        let contract_storage = EmbassySharedState::new(BTreeMap::new());
        let timer_sender = TIMER_CHANNELS[node_index].sender();
        let timer_receiver = TIMER_CHANNELS[node_index].receiver();

        spawner
            .spawn(semihosting_udp_node_task(
                node_index,
                node,
                slot.timer_state,
                slot.transport_state,
                slot.inbound_receiver,
                timer_receiver,
                height.clone(),
                contract_storage.clone(),
                cancel,
            ))
            .unwrap();

        Self {
            timer_sender,
            height,
            contract_storage,
            last_cert: cert_state,
            wal: wal_state,
        }
    }
}

#[embassy_executor::task(pool_size = 5)]
async fn semihosting_udp_node_task(
    node_index: usize,
    mut node: EtheramNode<IbftMessage>,
    timer_state: EmbassySharedState<InMemoryTimerState>,
    transport_state: EmbassySharedState<InMemoryTransportState<IbftMessage>>,
    inbound_receiver: IbftMessageReceiver,
    timer_receiver: TimerReceiver,
    height: EmbassySharedState<Height>,
    contract_storage: EmbassySharedState<BTreeMap<(Address, Hash), Hash>>,
    cancel: &'static CancellationToken,
) {
    let peer_id = node_index as PeerId;
    let ei_notify = udp_ei_notify_receiver(node_index);
    loop {
        match select4(
            cancel.wait(),
            inbound_receiver.receive(),
            timer_receiver.receive(),
            ei_notify.receive(),
        )
        .await
        {
            Either4::First(()) => break,
            Either4::Second((from, msg)) => {
                transport_state.with_mut(|s| s.push_message(peer_id, from, msg));
                while node.step() {}
                height.with_mut(|h| *h = node.state().query_height());
                contract_storage
                    .with_mut(|entries| *entries = node.state().snapshot_contract_storage());
            }
            Either4::Third(timer_event) => {
                timer_state.with_mut(|s| s.push_event(peer_id, timer_event));
                while node.step() {}
                height.with_mut(|h| *h = node.state().query_height());
                contract_storage
                    .with_mut(|entries| *entries = node.state().snapshot_contract_storage());
            }
            Either4::Fourth(()) => {
                while node.step() {}
                height.with_mut(|h| *h = node.state().query_height());
                contract_storage
                    .with_mut(|entries| *entries = node.state().snapshot_contract_storage());
            }
        }
    }
}
