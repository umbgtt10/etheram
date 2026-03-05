// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::infrastructure::RealInfrastructure;
use super::node_infra_slot::NodeInfraSlot;
use crate::cancellation_token::CancellationToken;
use crate::config::MAX_NODES;
use crate::embassy_shared_state::EmbassySharedState;
use crate::infra::external_interface::udp::udp_raft_external_interface::udp_raft_ei_notify_receiver;
use crate::infra::external_interface::udp::udp_raft_external_interface::UdpRaftExternalInterface;
use crate::infra::timer::channel::timer_channels::TIMER_CHANNELS;
use crate::infra::transport::udp::udp_raft_transport::RaftMessageReceiver;
use crate::infra::transport::udp::udp_raft_transport_hub::UdpRaftTransportHub;
use crate::raft_observer::RaftSemihostingObserver;
use crate::spawned_node::SpawnedNode;
use crate::spawned_node::TimerReceiver;
use alloc::boxed::Box;
use alloc::vec::Vec;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_futures::select::select4;
use embassy_futures::select::Either4;
use etheram_core::node_common::shared_state::SharedState;
use etheram_core::types::PeerId;
use raft_node::common_types::node_role::NodeRole;
use raft_node::executor::outgoing::outgoing_sources::RaftOutgoingSources;
use raft_node::executor::raft_executor::RaftExecutor;
use raft_node::implementations::eager_raft_context_builder::EagerRaftContextBuilder;
use raft_node::implementations::in_memory_raft_cache::InMemoryRaftCache;
use raft_node::implementations::in_memory_raft_state_machine::InMemoryRaftStateMachine;
use raft_node::implementations::in_memory_raft_timer::InMemoryRaftTimer;
use raft_node::implementations::in_memory_raft_timer::InMemoryRaftTimerState;
use raft_node::implementations::in_memory_raft_transport::InMemoryRaftTransport;
use raft_node::implementations::in_memory_raft_transport::InMemoryRaftTransportState;
use raft_node::implementations::raft::raft_protocol::RaftProtocol;
use raft_node::implementations::type_based_raft_partitioner::TypeBasedRaftPartitioner;
use raft_node::incoming::incoming_sources::RaftIncomingSources;
use raft_node::observer::RaftEventLevel;
use raft_node::raft_node::RaftNode;
use raft_node::state::raft_state::RaftState;

type P = Vec<u8>;

pub async fn init(spawner: &Spawner) -> RealInfrastructure {
    let mut transports = UdpRaftTransportHub::initialize(spawner).await;
    let slots = core::array::from_fn(|i| {
        let (inbound, outbound) = transports.remove(0);
        Some(NodeInfraSlot::new(
            i as PeerId,
            inbound.into_receiver(),
            outbound,
        ))
    });
    RealInfrastructure::new(slots)
}

struct NodeTaskContext {
    transport_state: EmbassySharedState<InMemoryRaftTransportState<P>>,
    timer_state: EmbassySharedState<InMemoryRaftTimerState>,
    commit_index: EmbassySharedState<u64>,
    term: EmbassySharedState<u64>,
    role: EmbassySharedState<NodeRole>,
}

impl SpawnedNode {
    pub fn new(
        spawner: &Spawner,
        node_index: usize,
        slot: NodeInfraSlot,
        cancel: &'static CancellationToken,
        node_cancel: &'static CancellationToken,
    ) -> Self {
        let peer_id = node_index as PeerId;
        let validators: Vec<PeerId> = (0..MAX_NODES as u64).collect();
        let peers: Vec<PeerId> = validators
            .iter()
            .filter(|&&p| p != peer_id)
            .copied()
            .collect();

        let incoming = RaftIncomingSources::new(
            Box::new(InMemoryRaftTimer::new(peer_id, slot.timer_state.clone())),
            Box::new(UdpRaftExternalInterface::new(node_index)),
            Box::new(InMemoryRaftTransport::new(
                peer_id,
                slot.transport_state.clone(),
            )),
        );

        let state = RaftState::new(Box::new(slot.storage), Box::new(InMemoryRaftCache::new()));

        let outgoing = RaftOutgoingSources::new(
            Box::new(InMemoryRaftTimer::new(peer_id, slot.timer_state.clone())),
            Box::new(UdpRaftExternalInterface::new(node_index)),
            Box::new(slot.outbound),
        );

        let executor = RaftExecutor::new_with_peers(outgoing, peers.clone());

        let node = RaftNode::new(
            peer_id,
            incoming,
            state,
            executor,
            Box::new(EagerRaftContextBuilder::new(peer_id, peers)),
            Box::new(RaftProtocol::<P>::new()),
            Box::new(TypeBasedRaftPartitioner::new()),
            Box::new(InMemoryRaftStateMachine::new()),
            Box::new(RaftSemihostingObserver::new(RaftEventLevel::Essential)),
        );

        let commit_index = EmbassySharedState::new(0u64);
        let term = EmbassySharedState::new(0u64);
        let role = EmbassySharedState::new(NodeRole::Follower);
        let timer_sender = TIMER_CHANNELS[node_index].sender();
        let timer_receiver = TIMER_CHANNELS[node_index].receiver();

        spawner
            .spawn(semihosting_udp_node_task(
                node_index,
                node,
                NodeTaskContext {
                    transport_state: slot.transport_state,
                    timer_state: slot.timer_state,
                    commit_index: commit_index.clone(),
                    term: term.clone(),
                    role: role.clone(),
                },
                slot.inbox_receiver,
                timer_receiver,
                cancel,
                node_cancel,
            ))
            .unwrap();

        Self {
            timer_sender,
            commit_index,
            term,
            role,
        }
    }
}

#[embassy_executor::task(pool_size = 5)]
async fn semihosting_udp_node_task(
    node_index: usize,
    mut node: RaftNode<P>,
    ctx: NodeTaskContext,
    inbox_receiver: RaftMessageReceiver,
    timer_receiver: TimerReceiver,
    cancel: &'static CancellationToken,
    node_cancel: &'static CancellationToken,
) {
    let peer_id = node_index as PeerId;
    let ei_notify = udp_raft_ei_notify_receiver(node_index);
    loop {
        match select4(
            async {
                select(cancel.wait(), node_cancel.wait()).await;
            },
            inbox_receiver.receive(),
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
                sync_state(&ctx, &node);
            }
            Either4::Third(timer_event) => {
                ctx.timer_state
                    .with_mut(|s| s.push_event(peer_id, timer_event));
                while node.step() {}
                sync_state(&ctx, &node);
            }
            Either4::Fourth(()) => {
                while node.step() {}
                sync_state(&ctx, &node);
            }
        }
    }
}

fn sync_state(ctx: &NodeTaskContext, node: &RaftNode<P>) {
    ctx.commit_index
        .with_mut(|c| *c = node.state().query_commit_index());
    ctx.term
        .with_mut(|t| *t = node.state().query_current_term());
    ctx.role.with_mut(|r| *r = node.state().query_role());
}
