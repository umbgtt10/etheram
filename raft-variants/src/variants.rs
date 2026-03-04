// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftStorageVariant {
    InMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftCacheVariant {
    InMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftTransportVariant {
    InMemory,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftTimerVariant {
    InMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftExternalInterfaceVariant {
    InMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftProtocolVariant {
    Raft,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftPartitionerVariant {
    TypeBased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftContextBuilderVariant {
    Eager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftObserverVariant {
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftStateMachineVariant {
    InMemory,
}
