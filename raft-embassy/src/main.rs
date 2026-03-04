// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![no_std]
#![no_main]

extern crate alloc;
extern crate panic_semihosting;

mod cancellation_token;
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

use crate::cancellation_token::CancellationToken;
use crate::raft_client::RaftClient;
use alloc::vec;
use cortex_m::Peripherals;
use cortex_m_semihosting::debug;
use embassy_executor::Spawner;
use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Timer;
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
    client.wait_for_leader(Duration::from_secs(3)).await;
    let leader_after_election = client.find_leader();
    let term_after_election = client.node_term(0);
    info!(
        "act0 leader: {:?} term: {}",
        leader_after_election, term_after_election
    );

    info!("=== Act 1: Replication ===");
    let cmd1 = encode_kv(b"counter", b"1");
    client.submit_command_all(1, cmd1);
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act1 submit: {:?}", response),
        Err(_) => info!("act1 submit timed out"),
    }
    let ci_before = client.node_commit_index(0);
    client
        .wait_for_commit_above(0, ci_before, Duration::from_secs(3))
        .await;
    let ci_after = client.node_commit_index(0);
    info!("act1 commit_index (expect >{}): {}", ci_before, ci_after);

    info!("=== Act 2: Read-after-write ===");
    let leader_index = client.find_leader().unwrap_or(0);
    client.submit_query(leader_index, 2, alloc::string::String::from("counter"));
    match with_timeout(Duration::from_secs(1), client.await_response(leader_index)).await {
        Ok((_, response)) => info!("act2 query counter (expect Applied([49])): {:?}", response),
        Err(_) => info!("act2 query timed out"),
    }

    info!("=== Act 3: Re-election ===");
    let leader_before = client.find_leader().unwrap_or(0);
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
    let new_leader = client.wait_for_leader(Duration::from_secs(3)).await;
    let new_term = client.node_term(challenger);
    info!("act3 new leader: {:?} term: {}", new_leader, new_term);

    info!("=== Act 4: Continued replication under new leader ===");
    let leader_idx = new_leader.unwrap_or(challenger);
    let cmd4 = encode_kv(b"counter", b"2");
    client.submit_command_all(3, cmd4);
    match with_timeout(Duration::from_secs(1), client.await_response(leader_idx)).await {
        Ok((_, response)) => info!("act4 submit: {:?}", response),
        Err(_) => info!("act4 submit timed out"),
    }
    let ci_before_act4 = client.node_commit_index(leader_idx);
    client
        .wait_for_commit_above(leader_idx, ci_before_act4, Duration::from_secs(3))
        .await;
    let ci_after_act4 = client.node_commit_index(leader_idx);
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
