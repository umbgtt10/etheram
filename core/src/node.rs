// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cache::Cache;
use crate::consensus_protocol::ConsensusProtocol;
use crate::external_interface_incoming::ExternalInterfaceIncoming;
use crate::external_interface_outgoing::ExternalInterfaceOutgoing;
use crate::storage::Storage;
use crate::timer_input::TimerInput;
use crate::timer_output::TimerOutput;
use crate::transport_incoming::TransportIncoming;
use crate::transport_outgoing::TransportOutgoing;

pub trait Node {
    type Protocol: ConsensusProtocol;

    type StorageQuery;
    type StorageMutation;
    type StorageQueryResult;
    type Storage: Storage<
        Query = Self::StorageQuery,
        Mutation = Self::StorageMutation,
        QueryResult = Self::StorageQueryResult,
    >;

    type CacheQuery;
    type CacheUpdate;
    type CacheQueryResult;
    type Cache: Cache<
        Query = Self::CacheQuery,
        Update = Self::CacheUpdate,
        QueryResult = Self::CacheQueryResult,
    >;

    type PeerMessage;
    type Transport: TransportIncoming<Message = Self::PeerMessage>
        + TransportOutgoing<Message = Self::PeerMessage>;

    type ClientRequest;
    type ClientResponse;
    type ExternalInterface: ExternalInterfaceIncoming<Request = Self::ClientRequest>
        + ExternalInterfaceOutgoing<Response = Self::ClientResponse>;

    type TimerEvent;
    type Timer: TimerInput<Event = Self::TimerEvent> + TimerOutput<Event = Self::TimerEvent>;

    fn step(&mut self) -> bool;
}
