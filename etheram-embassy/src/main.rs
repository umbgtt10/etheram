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
mod etheram_client;
mod heap;
mod infra;
mod logging;
mod semihosting_observer;
mod spawned_node;
mod time_driver;

use embassy_core::cancellation_token::CancellationToken;
pub use embassy_core::info;

use crate::config::MAX_NODES;
use crate::etheram_client::EtheramClient;
use cortex_m::Peripherals;
use cortex_m_semihosting::debug;
use embassy_executor::Spawner;
use embassy_time::with_timeout;
use embassy_time::Duration;
use embassy_time::Timer;
use etheram_node::common_types::transaction::Transaction;
use etheram_node::common_types::types::Address;
use etheram_node::common_types::types::Hash;
use etheram_node::implementations::ibft::consensus_wal::ConsensusWal;
use etheram_node::implementations::tiny_evm_engine::OPCODE_PUSH1;
use etheram_node::implementations::tiny_evm_engine::OPCODE_RETURN;
use etheram_node::implementations::tiny_evm_engine::OPCODE_SSTORE;
use etheram_node::implementations::tiny_evm_gas::GAS_PUSH1;
use etheram_node::implementations::tiny_evm_gas::GAS_SSTORE_SET;
use etheram_node::implementations::tiny_evm_gas::INTRINSIC_GAS;
use etheram_node::incoming::external_interface::client_request::ClientRequest;
use etheram_node::incoming::timer::timer_event::TimerEvent;

static CANCEL: CancellationToken = CancellationToken::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    heap::init();
    let mut cp = Peripherals::take().unwrap();
    time_driver::init(&mut cp.SYST);
    logging::init();

    let mut infra = configurations::init(&spawner).await;

    let node_0 = infra.create_node(&spawner, 0, &CANCEL);
    let node_1 = infra.create_node(&spawner, 1, &CANCEL);
    let node_2 = infra.create_node(&spawner, 2, &CANCEL);
    let node_3 = infra.create_node(&spawner, 3, &CANCEL);
    let node_4 = infra.create_node(&spawner, 4, &CANCEL);

    let client = EtheramClient::new(&CANCEL, [node_0, node_1, node_2, node_3, node_4]);

    let sender: Address = [1u8; 20];
    let receiver: Address = [2u8; 20];
    let act10_sender: Address = [3u8; 20];
    let act11_sender: Address = [4u8; 20];

    info!("=== Act 0: Warmup - verify IBFT is running ===");
    let h0 = client.node_height(0);
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, h0, Duration::from_secs(2))
        .await;
    let warmup_height = client.node_height(0);
    info!("warmup height: {}", warmup_height);

    info!("=== Act 1: Submit transfer, commit block, verify balances ===");
    let tx1 = Transaction::transfer(sender, receiver, 300, 21_000, 0);
    client.submit_to_all_nodes(0, ClientRequest::SubmitTransaction(tx1));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act1 submit tx: {:?}", response),
        Err(_) => info!("act1 submit tx timed out"),
    }
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, warmup_height, Duration::from_secs(2))
        .await;
    client.submit_request(0, 1, ClientRequest::GetHeight);
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act1 height: {:?}", response),
        Err(_) => info!("act1 height timed out"),
    }
    client.submit_request(0, 2, ClientRequest::GetBalance(sender));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act1 sender balance (expect 700): {:?}", response),
        Err(_) => info!("act1 sender balance timed out"),
    }
    client.submit_request(0, 3, ClientRequest::GetBalance(receiver));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act1 receiver balance (expect 500): {:?}", response),
        Err(_) => info!("act1 receiver balance timed out"),
    }

    info!("=== Act 2: Reverse transfer, verify cumulative balances ===");
    let tx2 = Transaction::transfer(receiver, sender, 200, 21_000, 0);
    client.submit_to_all_nodes(0, ClientRequest::SubmitTransaction(tx2));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act2 submit tx: {:?}", response),
        Err(_) => info!("act2 submit tx timed out"),
    }
    let h2 = client.node_height(0);
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, h2, Duration::from_secs(2))
        .await;
    client.submit_request(0, 4, ClientRequest::GetBalance(sender));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act2 sender balance (expect 900): {:?}", response),
        Err(_) => info!("act2 sender balance timed out"),
    }
    client.submit_request(0, 5, ClientRequest::GetBalance(receiver));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act2 receiver balance (expect 300): {:?}", response),
        Err(_) => info!("act2 receiver balance timed out"),
    }

    info!("=== Act 3: Overdraft rejection ===");
    let tx3 = Transaction::transfer(receiver, sender, 400, 21_000, 1);
    client.submit_request(0, 6, ClientRequest::SubmitTransaction(tx3));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!(
            "act3 overdraft (expect InsufficientBalance): {:?}",
            response
        ),
        Err(_) => info!("act3 overdraft timed out"),
    }

    info!("=== Act 4: View change via TimeoutRound ===");
    let h4 = client.node_height(0);
    client.fire_timer_all(TimerEvent::TimeoutRound).await;
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, h4, Duration::from_secs(2))
        .await;
    let height_after_view_change = client.node_height(0);
    info!(
        "act4 height after view change: {}",
        height_after_view_change
    );

    info!("=== Act 5: Stale nonce rejection ===");
    let tx5 = Transaction::transfer(sender, receiver, 100, 21_000, 0);
    client.submit_request(0, 7, ClientRequest::SubmitTransaction(tx5));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act5 stale nonce (expect InvalidNonce): {:?}", response),
        Err(_) => info!("act5 stale nonce timed out"),
    }

    info!("=== Act 6: Gas limit exceeded ===");
    let tx6 = Transaction::transfer(sender, receiver, 100, 1_000_001, 1);
    client.submit_request(0, 8, ClientRequest::SubmitTransaction(tx6));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act6 gas limit (expect GasLimitExceeded): {:?}", response),
        Err(_) => info!("act6 gas limit timed out"),
    }

    info!("=== Act 7: Validator set update at height 5 ===");
    let height_after_vsu = client.node_height(0);
    info!(
        "act7 height after validator set update (expect >= 10): {}",
        height_after_vsu
    );

    info!("=== Act 8: WAL durable round-trip ===");
    if let Some(wal) = client.node_wal(0) {
        let bytes = wal.to_bytes();
        if let Some(restored) = ConsensusWal::from_bytes(&bytes) {
            info!("act8 wal height: {}", wal.height);
            info!(
                "act8 round-trip height (expect {}): {}",
                wal.height, restored.height
            );
        }
    }

    info!("=== Act 9: Ed25519 PreparedCertificate proof ===");
    let h9 = client.node_height(0);
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, h9, Duration::from_secs(2))
        .await;
    if let Some(cert) = client.node_last_cert(0) {
        let count = cert.signed_prepares.len();
        info!("act9 signed_prepares count (expect 4): {}", count);
        let nonzero = cert
            .signed_prepares
            .first()
            .is_some_and(|(_, sig)| sig.as_bytes().iter().any(|b| *b != 0));
        info!(
            "act9 first sig nonzero (expect false for mock, true for ed25519): {}",
            nonzero
        );
    }

    info!("=== Act 10: TinyEVM SSTORE contract storage demonstration ===");
    let h10 = client.node_height(0);
    let bytecode = alloc::vec![
        OPCODE_PUSH1,
        0x2a,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let tx10 = Transaction::new(
        act10_sender,
        receiver,
        0,
        INTRINSIC_GAS + 2 * GAS_PUSH1 + GAS_SSTORE_SET,
        0,
        bytecode,
    );
    client.submit_to_all_nodes(0, ClientRequest::SubmitTransaction(tx10));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!("act10 submit tx: {:?}", response),
        Err(_) => info!("act10 submit tx timed out"),
    }
    let slot_zero: Hash = [0u8; 32];
    client.fire_timer_all(TimerEvent::TimeoutRound).await;
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, h10, Duration::from_secs(2))
        .await;
    let mut storage_value = None;
    for _ in 0..50 {
        for node_index in 0..MAX_NODES {
            if let Some(value) = client.node_contract_storage(node_index, receiver, slot_zero) {
                storage_value = Some(value);
                break;
            }
        }
        if storage_value.is_some() {
            break;
        }
        Timer::after(Duration::from_millis(20)).await;
    }
    info!("act10 contract storage slot 0: {:?}", storage_value);

    info!("=== Act 11: TinyEVM gas failure demonstration (OutOfGas revert) ===");
    let h11 = client.node_height(0);
    let bytecode11 = alloc::vec![
        OPCODE_PUSH1,
        0x99,
        OPCODE_PUSH1,
        0x00,
        OPCODE_SSTORE,
        OPCODE_RETURN,
    ];
    let tx11 = Transaction::new(act11_sender, receiver, 0, 100, 0, bytecode11);
    client.submit_to_all_nodes(0, ClientRequest::SubmitTransaction(tx11));
    match with_timeout(Duration::from_secs(1), client.await_response(0)).await {
        Ok((_, response)) => info!(
            "act11 submit tx (expect TransactionAccepted): {:?}",
            response
        ),
        Err(_) => info!("act11 submit tx timed out"),
    }
    client.fire_timer_all(TimerEvent::ProposeBlock).await;
    client
        .wait_for_height_above(0, h11, Duration::from_secs(2))
        .await;
    let mut act11_storage = None;
    for _ in 0..50 {
        for node_index in 0..MAX_NODES {
            if let Some(value) = client.node_contract_storage(node_index, receiver, slot_zero) {
                act11_storage = Some(value);
                break;
            }
        }
        if act11_storage.is_some() {
            break;
        }
        Timer::after(Duration::from_millis(20)).await;
    }
    info!(
        "act11 slot 0 after OutOfGas (expect 0x2a from Act10, not 0x99): {:?}",
        act11_storage
    );

    client.shutdown();

    Timer::after(Duration::from_millis(500)).await;

    debug::exit(debug::EXIT_SUCCESS);
}
