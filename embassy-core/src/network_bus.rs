// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::task::Waker;
use embassy_net_driver::Capabilities;
use embassy_net_driver::Driver;
use embassy_net_driver::HardwareAddress;
use embassy_net_driver::LinkState;
use embassy_net_driver::RxToken;
use embassy_net_driver::TxToken;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;

const MTU: usize = 1500;
const CHANNEL_SIZE: usize = 16;

pub struct NetworkBus {
    queues: [Channel<CriticalSectionRawMutex, Packet, CHANNEL_SIZE>; 5],
    wakers: [Mutex<CriticalSectionRawMutex, RefCell<Option<Waker>>>; 5],
}

#[derive(Clone)]
pub struct Packet {
    pub data: Vec<u8>,
    pub len: usize,
}

impl NetworkBus {
    pub const fn new() -> Self {
        Self {
            queues: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
            ],
            wakers: [
                Mutex::new(RefCell::new(None)),
                Mutex::new(RefCell::new(None)),
                Mutex::new(RefCell::new(None)),
                Mutex::new(RefCell::new(None)),
                Mutex::new(RefCell::new(None)),
            ],
        }
    }

    fn node_queue(&self, index: usize) -> &Channel<CriticalSectionRawMutex, Packet, CHANNEL_SIZE> {
        &self.queues[index]
    }

    fn register_waker(&self, index: usize, waker: &Waker) {
        self.wakers[index].lock(|cell| {
            let mut w = cell.borrow_mut();
            let needs_update = match w.as_ref() {
                Some(old) => !old.will_wake(waker),
                None => true,
            };
            if needs_update {
                *w = Some(waker.clone());
            }
        });
    }

    fn wake_node(&self, index: usize) {
        self.wakers[index].lock(|cell| {
            if let Some(waker) = cell.borrow_mut().take() {
                waker.wake();
            }
        });
    }
}

impl Default for NetworkBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MockNetDriver {
    node_index: usize,
    mac_addr: [u8; 6],
    bus: &'static NetworkBus,
}

impl MockNetDriver {
    pub fn new(node_index: usize, bus: &'static NetworkBus) -> Self {
        Self {
            node_index,
            mac_addr: [0x02, 0x00, 0x00, 0x00, 0x00, (node_index + 1) as u8],
            bus,
        }
    }

    fn try_recv_packet(&self) -> Option<Packet> {
        self.bus.node_queue(self.node_index).try_receive().ok()
    }
}

impl Driver for MockNetDriver {
    type RxToken<'a>
        = MockRxToken
    where
        Self: 'a;

    type TxToken<'a>
        = MockTxToken<'a>
    where
        Self: 'a;

    fn receive(
        &mut self,
        cx: &mut core::task::Context,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if let Some(packet) = self.try_recv_packet() {
            let rx = MockRxToken {
                packet: Some(packet),
            };
            let tx = MockTxToken {
                node_index: self.node_index,
                bus: self.bus,
            };
            Some((rx, tx))
        } else {
            self.bus.register_waker(self.node_index, cx.waker());
            None
        }
    }

    fn transmit(&mut self, _cx: &mut core::task::Context) -> Option<Self::TxToken<'_>> {
        Some(MockTxToken {
            node_index: self.node_index,
            bus: self.bus,
        })
    }

    fn link_state(&mut self, _cx: &mut core::task::Context) -> LinkState {
        LinkState::Up
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(1);
        caps
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet(self.mac_addr)
    }
}

pub struct MockRxToken {
    packet: Option<Packet>,
}

impl RxToken for MockRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        if let Some(mut pkt) = self.packet {
            f(&mut pkt.data[..pkt.len])
        } else {
            f(&mut [])
        }
    }
}

pub struct MockTxToken<'a> {
    node_index: usize,
    bus: &'a NetworkBus,
}

impl<'a> TxToken for MockTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);
        if buffer.len() >= 6 {
            let dest_mac = [
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5],
            ];
            if dest_mac[0] == 0xFF {
                for target in 0..5 {
                    if target != self.node_index {
                        let packet = Packet {
                            data: buffer.clone(),
                            len,
                        };
                        if self.bus.node_queue(target).try_send(packet).is_ok() {
                            self.bus.wake_node(target);
                        }
                    }
                }
            } else {
                let target = (dest_mac[5] as usize).saturating_sub(1);
                if target < 5 {
                    let packet = Packet { data: buffer, len };
                    if self.bus.node_queue(target).try_send(packet).is_ok() {
                        self.bus.wake_node(target);
                    }
                }
            }
        }
        result
    }
}
