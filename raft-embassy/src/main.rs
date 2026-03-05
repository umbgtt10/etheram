// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]
#![no_main]

extern crate alloc;
extern crate panic_semihosting;

mod config;
mod configurations;
mod embassy_shared_state;
mod heap;
mod infra;
mod logging;
mod raft_client;
mod raft_observer;
mod spawned_node;
mod time_driver;

use embassy_core::cancellation_token::CancellationToken;
pub use embassy_core::info;

use crate::config::MAX_NODES;
use crate::raft_client::RaftClient;
use alloc::vec;
use cortex_m::Peripherals;
use cortex_m_semihosting::debug;
use embassy_executor::Spawner;
use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Timer;
use raft_node::executor::outgoing::external_interface::client_response::RaftClientResponse;
use raft_node::incoming::timer::timer_event::RaftTimerEvent;

static CANCEL: CancellationToken = CancellationToken::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    heap::init();

    let mut cp = Peripherals::take().unwrap();
    time_driver::init(&mut cp.SYST);

    logging::init();

    let mut infra = configurations::init(&spawner).await;

    let node_0 = infra.create_node(&spawner, 0, &CANCEL, &CANCEL);
    let node_1 = infra.create_node(&spawner, 1, &CANCEL, &CANCEL);
    let node_2 = infra.create_node(&spawner, 2, &CANCEL, &CANCEL);
    let node_3 = infra.create_node(&spawner, 3, &CANCEL, &CANCEL);
    let node_4 = infra.create_node(&spawner, 4, &CANCEL, &CANCEL);

    let client = RaftClient::new(&CANCEL, [node_0, node_1, node_2, node_3, node_4]);

    info!("=== Act 0: Election ===");
    client
        .fire_timer_to(0, RaftTimerEvent::ElectionTimeout)
        .await;
    let leader_after_election = elect_leader_or_panic(&client, Duration::from_secs(3)).await;
    let term_after_election = client.node_term(leader_after_election);
    if term_after_election == 0 {
        panic!("act0 failed: term_after_election must be > 0");
    }
    info!(
        "act0 leader: {:?} term: {}",
        Some(leader_after_election),
        term_after_election
    );

    info!("=== Act 1: Replication ===");
    let ci_before = client.node_commit_index(leader_after_election);
    let cmd1 = encode_kv(b"counter", b"1");
    client.submit_command_all(1, cmd1);
    let applied_node_act1 = await_applied_response_with_redirects(
        &client,
        leader_after_election,
        Duration::from_secs(1),
    )
    .await;
    info!("act1 submit: Applied via node {}", applied_node_act1);
    client
        .wait_for_commit_above(leader_after_election, ci_before, Duration::from_secs(3))
        .await;
    let ci_after = client.node_commit_index(leader_after_election);
    if ci_after <= ci_before {
        panic!("act1 failed: commit index did not advance");
    }
    info!("act1 commit_index (expect >{}): {}", ci_before, ci_after);

    info!("=== Act 2: Read-after-write ===");
    let leader_index = elect_leader_or_panic(&client, Duration::from_secs(2)).await;
    client.submit_query(leader_index, 2, alloc::string::String::from("counter"));
    match with_timeout(Duration::from_secs(1), client.await_response(leader_index)).await {
        Ok((_, RaftClientResponse::QueryResult(value))) => {
            if value != b"1".to_vec() {
                panic!("act2 failed: query returned unexpected value");
            }
            info!("act2 query counter (expect Applied([49])): QueryResult([49])");
        }
        Ok((_, response)) => panic!("act2 failed: unexpected query response: {:?}", response),
        Err(_) => panic!("act2 failed: query timed out"),
    }

    info!("=== Act 3: Re-election ===");
    let leader_before = elect_leader_or_panic(&client, Duration::from_secs(2)).await;
    let term_before = client.node_term(leader_before);
    info!("act3 leader {:?} at term {}", leader_before, term_before);
    let challenger = if leader_before == 0 { 1 } else { 0 };
    client
        .fire_timer_to(challenger, RaftTimerEvent::ElectionTimeout)
        .await;
    client
        .wait_for_term_above(challenger, term_before, Duration::from_secs(3))
        .await;
    client
        .fire_timer_to(challenger, RaftTimerEvent::ElectionTimeout)
        .await;
    let new_leader = elect_leader_or_panic(&client, Duration::from_secs(3)).await;
    let new_term = client.node_term(challenger);
    if new_term <= term_before {
        panic!("act3 failed: term did not advance");
    }
    info!("act3 new leader: {:?} term: {}", Some(new_leader), new_term);

    info!("=== Act 4: Continued replication under new leader ===");
    let leader_idx = new_leader;
    let ci_before_act4 = client.node_commit_index(leader_idx);
    let cmd4 = encode_kv(b"counter", b"2");
    client.submit_command_all(3, cmd4);
    let applied_node_act4 =
        await_applied_response_with_redirects(&client, leader_idx, Duration::from_secs(1)).await;
    info!("act4 submit: Applied via node {}", applied_node_act4);
    client
        .wait_for_commit_above(leader_idx, ci_before_act4, Duration::from_secs(3))
        .await;
    let ci_after_act4 = client.node_commit_index(leader_idx);
    if ci_after_act4 <= ci_before_act4 {
        panic!("act4 failed: commit index did not advance");
    }
    info!(
        "act4 commit_index (expect >{}): {}",
        ci_before_act4, ci_after_act4
    );

    info!("=== Full success! ===");

    client.shutdown();

    Timer::after(Duration::from_millis(500)).await;

    debug::exit(debug::EXIT_SUCCESS);
}

fn encode_kv(key: &[u8], value: &[u8]) -> alloc::vec::Vec<u8> {
    let mut cmd = vec![key.len() as u8];
    cmd.extend_from_slice(key);
    cmd.extend_from_slice(value);
    cmd
}

async fn elect_leader_or_panic(client: &RaftClient, timeout: Duration) -> usize {
    if let Some(leader) = client.wait_for_leader(timeout).await {
        return leader;
    }

    for node_index in 0..MAX_NODES {
        client
            .fire_timer_to(node_index, RaftTimerEvent::ElectionTimeout)
            .await;
        if let Some(leader) = client.wait_for_leader(timeout).await {
            return leader;
        }
    }

    panic!("leader election failed");
}

async fn await_applied_response_with_redirects(
    client: &RaftClient,
    start_node: usize,
    timeout: Duration,
) -> usize {
    let mut node_index = start_node;
    let mut redirects = 0usize;

    loop {
        let response = match with_timeout(timeout, client.await_response(node_index)).await {
            Ok((_, response)) => response,
            Err(_) => panic!(
                "submit failed: timed out waiting response from {}",
                node_index
            ),
        };

        match response {
            RaftClientResponse::Applied(_) => return node_index,
            RaftClientResponse::NotLeader(Some(next_leader)) => {
                let next_index = next_leader as usize;
                info!("submit redirected from {} to {}", node_index, next_index);
                node_index = next_index;
                redirects += 1;
                if redirects > (MAX_NODES * 3) {
                    panic!("submit failed: too many redirects");
                }
            }
            other => panic!("submit failed: unexpected response: {:?}", other),
        }
    }
}
