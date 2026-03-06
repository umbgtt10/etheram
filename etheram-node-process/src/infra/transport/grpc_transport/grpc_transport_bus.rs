// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_core::types::PeerId;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::OnceLock;

type TransportQueue = VecDeque<(PeerId, ())>;
type TransportBusState = BTreeMap<PeerId, TransportQueue>;

fn bus() -> &'static Mutex<TransportBusState> {
    static BUS: OnceLock<Mutex<TransportBusState>> = OnceLock::new();
    BUS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub fn dequeue_for(node_id: PeerId) -> Option<(PeerId, ())> {
    let mut guard = bus().lock().expect("transport bus lock poisoned");
    let queue = guard.entry(node_id).or_default();
    queue.pop_front()
}

pub fn enqueue_to(to_peer: PeerId, from_peer: PeerId) {
    let mut guard = bus().lock().expect("transport bus lock poisoned");
    let queue = guard.entry(to_peer).or_default();
    queue.push_back((from_peer, ()));
}
