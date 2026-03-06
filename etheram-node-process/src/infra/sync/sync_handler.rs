// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::infra::storage::in_memory_storage::InMemoryStorage;
use crate::infra::sync::sync_import::decode_and_validate_blocks;
use crate::infra::sync::sync_message::SyncMessage;
use crate::infra::sync::sync_sender::SyncSender;
use crate::infra::sync::sync_state::SyncState;
use crate::infra::transport::grpc_transport::sync_bus::SyncBus;
use crate::infra::transport::grpc_transport::wire_ibft_message::serialize_block;
use etheram_core::types::PeerId;
use etheram_node::state::etheram_state::EtheramState;
use std::sync::Arc;
use std::time::Duration;

const SYNC_REQUEST_TIMEOUT_MS: u64 = 1000;
const SYNC_MAX_RETRIES: u32 = 3;
const SYNC_MAX_BLOCKS_PER_REQUEST: u64 = 64;

pub struct SyncHandler {
    sync_bus: Arc<SyncBus>,
    sync_sender: Box<dyn SyncSender>,
    sync_state: SyncState,
    sync_storage: InMemoryStorage,
}

impl SyncHandler {
    pub fn new(
        sync_bus: Arc<SyncBus>,
        sync_sender: Box<dyn SyncSender>,
        sync_storage: InMemoryStorage,
    ) -> Self {
        Self {
            sync_bus,
            sync_sender,
            sync_state: SyncState::new(),
            sync_storage,
        }
    }

    pub fn process_sync_messages(&mut self, peer_id: PeerId, state: &EtheramState) {
        while let Some((from_peer, message)) = self.sync_bus.dequeue_sync_for(peer_id) {
            match message {
                SyncMessage::Status { height, .. } => {
                    self.sync_state.observe_status(from_peer, height);
                    let local_height = state.query_height();
                    if let Some(lag_distance) = self.sync_state.lag_distance(local_height) {
                        println!(
                            "sync_state peer_id={} lagging_by={} best_peer_height={}",
                            peer_id,
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
                    self.handle_get_blocks(from_peer, from_height, max_blocks, state);
                }
                SyncMessage::Blocks {
                    start_height,
                    block_payloads,
                } => {
                    self.handle_blocks(peer_id, from_peer, start_height, &block_payloads, state);
                }
            }
        }
    }

    pub fn process_sync_timeouts(&mut self, peer_id: PeerId, local_height: u64) {
        if let Some((target_peer, from_height, max_blocks)) =
            self.sync_state.handle_request_timeout(
                local_height,
                Duration::from_millis(SYNC_REQUEST_TIMEOUT_MS),
                SYNC_MAX_RETRIES,
            )
        {
            self.sync_sender.send_to_peer(
                target_peer,
                &SyncMessage::GetBlocks {
                    from_height,
                    max_blocks,
                },
            );
            println!(
                "sync_timeout_retry peer_id={} target_peer={} from_height={} max_blocks={}",
                peer_id, target_peer, from_height, max_blocks
            );
        }
    }

    pub fn broadcast_status(&self, height: u64, last_hash: [u8; 32]) {
        self.sync_sender.broadcast_status(height, last_hash);
    }

    fn handle_get_blocks(
        &mut self,
        from_peer: PeerId,
        from_height: u64,
        max_blocks: u64,
        state: &EtheramState,
    ) {
        let mut block_payloads = Vec::new();
        for offset in 0..max_blocks {
            let height = from_height + offset;
            let Some(block) = state.query_block(height) else {
                break;
            };
            let Ok(payload) = serialize_block(&block) else {
                break;
            };
            block_payloads.push(payload);
        }

        self.sync_sender.send_to_peer(
            from_peer,
            &SyncMessage::Blocks {
                start_height: from_height,
                block_payloads,
            },
        );
    }

    fn handle_blocks(
        &mut self,
        peer_id: PeerId,
        from_peer: PeerId,
        start_height: u64,
        block_payloads: &[Vec<u8>],
        state: &EtheramState,
    ) {
        let local_height = state.query_height();
        let expected_parent_post_state_root = if local_height == 0 {
            None
        } else {
            state
                .query_block(local_height - 1)
                .map(|block| block.post_state_root)
        };
        let decoded_blocks = decode_and_validate_blocks(
            local_height,
            start_height,
            block_payloads,
            expected_parent_post_state_root,
        );
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
                .complete_in_flight_request(from_peer, start_height)
        };

        let failed = if malformed || empty_while_lagging {
            self.sync_state
                .fail_in_flight_request(from_peer, start_height)
        } else {
            false
        };

        if completed || failed {
            if let Some((target_peer, from_height, max_blocks)) = self
                .sync_state
                .next_request(state.query_height(), SYNC_MAX_BLOCKS_PER_REQUEST)
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
            peer_id,
            from_peer,
            start_height,
            block_payloads.len(),
            decoded,
            malformed,
            empty_while_lagging,
            completed,
            failed
        );
    }
}
