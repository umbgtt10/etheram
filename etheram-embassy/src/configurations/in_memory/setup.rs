// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::infrastructure::InMemoryInfrastructure;
use super::node_infra_slot::NodeInfraSlot;
use super::node_infra_slot::OutboxState;
use crate::cancellation_token::CancellationToken;
use crate::config::MAX_NODES;
use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::external_interface::channel::channel_external_interface::ChannelExternalInterface;
use crate::infra::external_interface::channel::client_request_hub::ei_notify_receiver;
use crate::infra::storage::captured_wal_writer::CapturedWalWriter;
use crate::infra::timer::channel::timer_channels::TIMER_CHANNELS;
use crate::infra::transport::channel::channel_transport_hub::TRANSPORT_HUB;
use crate::infra::transport::channel::outbox_transport::OutboxTransport;
use crate::semihosting_observer::SemihostingObserver;
use crate::spawned_node::SpawnedNode;
use crate::spawned_node::TimerReceiver;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use barechain_core::types::PeerId;
use barechain_etheram_variants::implementations::eager_context_builder::EagerContextBuilder;
use barechain_etheram_variants::implementations::ibft::consensus_wal::ConsensusWal;
use barechain_etheram_variants::implementations::ibft::ibft_message::IbftMessage;
use barechain_etheram_variants::implementations::ibft::ibft_protocol::IbftProtocol;
use barechain_etheram_variants::implementations::ibft::mock_signature_scheme::MockSignatureScheme;
use barechain_etheram_variants::implementations::ibft::prepared_certificate::PreparedCertificate;
use barechain_etheram_variants::implementations::ibft::validator_set_update::ValidatorSetUpdate;
use barechain_etheram_variants::implementations::in_memory_cache::InMemoryCache;
use barechain_etheram_variants::implementations::in_memory_timer::InMemoryTimer;
use barechain_etheram_variants::implementations::in_memory_timer::InMemoryTimerState;
use barechain_etheram_variants::implementations::in_memory_transport::InMemoryTransport;
use barechain_etheram_variants::implementations::in_memory_transport::InMemoryTransportState;
use barechain_etheram_variants::implementations::shared_state::SharedState;
use barechain_etheram_variants::implementations::tiny_evm_engine::TinyEvmEngine;
use barechain_etheram_variants::implementations::type_based_partitioner::TypeBasedPartitioner;
use embassy_executor::Spawner;
use embassy_futures::select::select4;
use embassy_futures::select::Either4;
use etheram::common_types::types::Address;
use etheram::common_types::types::Hash;
use etheram::common_types::types::Height;
use etheram::etheram_node::EtheramNode;
use etheram::executor::etheram_executor::EtheramExecutor;
use etheram::executor::outgoing::outgoing_sources::OutgoingSources;
use etheram::incoming::incoming_sources::IncomingSources;
use etheram::observer::EventLevel;
use etheram::state::etheram_state::EtheramState;

struct NodeTaskContext {
    transport_state: EmbassySharedState<InMemoryTransportState<IbftMessage>>,
    timer_state: EmbassySharedState<InMemoryTimerState>,
    outbox_state: OutboxState,
    height: EmbassySharedState<Height>,
    contract_storage: EmbassySharedState<BTreeMap<(Address, Hash), Hash>>,
}

pub async fn init(_spawner: &Spawner) -> InMemoryInfrastructure {
    let sender: Address = [1u8; 20];
    let receiver: Address = [2u8; 20];
    let act10_sender: Address = [3u8; 20];
    InMemoryInfrastructure::new(core::array::from_fn(|_| {
        Some(
            NodeInfraSlot::new()
                .with_genesis_account(sender, 1_000)
                .with_genesis_account(receiver, 200)
                .with_genesis_account(act10_sender, 1_000),
        )
    }))
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
            Box::new(ChannelExternalInterface::new(node_index)),
            Box::new(InMemoryTransport::new(
                peer_id,
                slot.transport_state.clone(),
            )),
        );

        let state = EtheramState::new(Box::new(slot.storage), Box::new(InMemoryCache::new()));

        let outgoing = OutgoingSources::new(
            Box::new(InMemoryTimer::new(peer_id, slot.timer_state.clone())),
            Box::new(ChannelExternalInterface::new(node_index)),
            Box::new(OutboxTransport::new(slot.outbox_state.clone())),
        );

        let executor = EtheramExecutor::new_with_peers(outgoing, peers);

        let wal_state: EmbassySharedState<Option<ConsensusWal>> = EmbassySharedState::new(None);
        let cert_state: EmbassySharedState<Option<PreparedCertificate>> =
            EmbassySharedState::new(None);
        let wal_writer = Box::new(CapturedWalWriter::new(
            wal_state.clone(),
            cert_state.clone(),
        ));

        let node = EtheramNode::new(
            peer_id,
            incoming,
            state,
            executor,
            Box::new(EagerContextBuilder::new()),
            Box::new(
                IbftProtocol::new_with_validator_updates(
                    validators,
                    Box::new(MockSignatureScheme::new(peer_id)),
                    alloc::vec![ValidatorSetUpdate::new(5, (0..MAX_NODES as u64).collect())],
                )
                .with_wal_writer(wal_writer),
            ),
            Box::new(TypeBasedPartitioner::new()),
            Box::new(TinyEvmEngine),
            Box::new(SemihostingObserver::new(EventLevel::Essential)),
        );

        let height = EmbassySharedState::new(0u64);
        let contract_storage = EmbassySharedState::new(BTreeMap::new());
        let timer_sender = TIMER_CHANNELS[node_index].sender();
        let timer_receiver = TIMER_CHANNELS[node_index].receiver();

        spawner
            .spawn(in_memory_channel_node_task(
                node_index,
                node,
                NodeTaskContext {
                    transport_state: slot.transport_state,
                    timer_state: slot.timer_state,
                    outbox_state: slot.outbox_state,
                    height: height.clone(),
                    contract_storage: contract_storage.clone(),
                },
                timer_receiver,
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
async fn in_memory_channel_node_task(
    node_index: usize,
    mut node: EtheramNode<IbftMessage>,
    ctx: NodeTaskContext,
    timer_receiver: TimerReceiver,
    cancel: &'static CancellationToken,
) {
    let peer_id = node_index as PeerId;
    let ei_notify = ei_notify_receiver(node_index);
    loop {
        match select4(
            cancel.wait(),
            TRANSPORT_HUB.receive(node_index),
            timer_receiver.receive(),
            ei_notify.receive(),
        )
        .await
        {
            Either4::First(()) => break,
            Either4::Second((from, msg)) => {
                ctx.transport_state
                    .with_mut(|s| s.push_message(peer_id, from, msg));
                while node.step() {}
                flush_outbox(&ctx.outbox_state, peer_id).await;
                ctx.height.with_mut(|h| *h = node.state().query_height());
                ctx.contract_storage
                    .with_mut(|entries| *entries = node.state().snapshot_contract_storage());
            }
            Either4::Third(timer_event) => {
                ctx.timer_state
                    .with_mut(|s| s.push_event(peer_id, timer_event));
                while node.step() {}
                flush_outbox(&ctx.outbox_state, peer_id).await;
                ctx.height.with_mut(|h| *h = node.state().query_height());
                ctx.contract_storage
                    .with_mut(|entries| *entries = node.state().snapshot_contract_storage());
            }
            Either4::Fourth(()) => {
                while node.step() {}
                flush_outbox(&ctx.outbox_state, peer_id).await;
                ctx.height.with_mut(|h| *h = node.state().query_height());
                ctx.contract_storage
                    .with_mut(|entries| *entries = node.state().snapshot_contract_storage());
            }
        }
    }
}

async fn flush_outbox(outbox_state: &OutboxState, from_peer: PeerId) {
    let messages = outbox_state.with_mut(core::mem::take);
    for (to, msg) in messages {
        TRANSPORT_HUB.send(to as usize, from_peer, msg).await;
    }
}
