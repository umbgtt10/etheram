// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use raft_node::incoming::timer::timer_event::RaftTimerEvent;
use raft_validation::raft_cluster::RaftCluster;

pub fn setup_elected_3_node_cluster() -> (RaftCluster, usize) {
    let mut cluster = RaftCluster::new(3);
    let leader_idx = cluster.elect_leader();
    (cluster, leader_idx)
}

pub fn setup_elected_5_node_cluster() -> (RaftCluster, usize) {
    let mut cluster = RaftCluster::new(5);
    let leader_idx = cluster.elect_leader();
    (cluster, leader_idx)
}

pub fn make_kv_command(key: &str, value: &[u8]) -> Vec<u8> {
    let key_bytes = key.as_bytes();
    let mut cmd = Vec::new();
    cmd.push(key_bytes.len() as u8);
    cmd.extend_from_slice(key_bytes);
    cmd.extend_from_slice(value);
    cmd
}

pub fn replicate_one(cluster: &mut RaftCluster, leader_idx: usize, payload: Vec<u8>) {
    cluster.submit_command(leader_idx, 1, payload);
    cluster.drain_all();
    cluster.fire_timer(leader_idx, RaftTimerEvent::Heartbeat);
    cluster.drain_all();
}
