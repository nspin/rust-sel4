//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use heapless::Vec;
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use smoltcp::wire::ETHERNET_HEADER_LEN;

use sddf::net::{BUFFER_SIZE, BufferDescriptor, Drainer, Filler, QueueHandle};
use sddf::resources::Region;
use sddf::sys::net_client_config_t;
use sel4_shared_memory::SharedMemoryRef;
use sel4_shared_memory::access::ReadOnly;

/// The IP MTU.
///
/// sDDF provides no means of choosing or discovering an MTU. Like sDDF's own lwIP-based clients
/// (see `SDDF_LWIP_ETHER_MTU`), this assumes standard Ethernet.
const IP_MTU: usize = 1500;

/// The maximum Ethernet frame size, excluding the frame check sequence. For Ethernet, this is what
/// smoltcp calls the MTU.
const MAX_FRAME_SIZE: usize = IP_MTU + ETHERNET_HEADER_LEN;

const _: () = assert!(MAX_FRAME_SIZE <= BUFFER_SIZE);

pub struct DeviceImpl {
    rx: Rx,
    tx: Tx,
}

// Frames are copied between shared memory and private buffers, so that smoltcp is never given
// references to shared memory.

struct Rx {
    queue: QueueHandle<Drainer>,
    data: SharedMemoryRef<'static, [u8], ReadOnly>,
}

struct Tx {
    queue: QueueHandle<Filler>,
    data: SharedMemoryRef<'static, [u8]>,
    reserved: Option<BufferDescriptor>,
}

impl DeviceImpl {
    /// # Safety
    ///
    /// The queues and data regions described by `config` must be valid and shared only with a
    /// peer which follows the sDDF network protocol.
    pub unsafe fn new(config: &net_client_config_t) -> Self {
        let mut this = unsafe {
            Self {
                rx: Rx {
                    queue: QueueHandle::from_connection_resource(&config.rx),
                    data: SharedMemoryRef::new_read_only(
                        Region::from_resource(&config.rx_data).unwrap().slice(),
                    ),
                },
                tx: Tx {
                    queue: QueueHandle::from_connection_resource(&config.tx),
                    data: SharedMemoryRef::new(
                        Region::from_resource(&config.tx_data).unwrap().slice(),
                    ),
                    reserved: None,
                },
            }
        };
        this.tx.queue.buffers_init(0);
        this
    }
}

impl Rx {
    fn receive(&mut self) -> Option<Vec<u8, BUFFER_SIZE>> {
        let mut buffer = self.queue.dequeue_active_or_request_signal()?;
        let range = buffer.range();
        let mut frame = Vec::new();
        frame.resize(range.len(), 0).unwrap();
        self.data.as_ptr().index(range).copy_into_slice(&mut frame);
        buffer.set_length(0);
        self.queue.enqueue_free(buffer).unwrap();
        self.queue.signal_free();
        Some(frame)
    }
}

impl Tx {
    fn reserve(&mut self) -> bool {
        if self.reserved.is_none() {
            self.reserved = self.queue.dequeue_free_or_request_signal();
        }
        self.reserved.is_some()
    }

    fn send<T>(&mut self, len: usize, f: impl FnOnce(&mut [u8]) -> T) -> T {
        let mut buffer = self.reserved.take().unwrap();
        buffer.set_length(len);
        let mut frame = Vec::<u8, BUFFER_SIZE>::new();
        frame.resize(len, 0).unwrap();
        let r = f(&mut frame);
        self.data
            .as_mut_ptr()
            .index(buffer.range())
            .copy_from_slice(&frame);
        self.queue.enqueue_active(buffer).unwrap();
        self.queue.signal_active();
        r
    }
}

impl Device for DeviceImpl {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = MAX_FRAME_SIZE;
        caps
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if !self.tx.reserve() {
            return None;
        }
        // If nothing has been received, the reserved Tx buffer simply stays reserved
        let frame = self.rx.receive()?;
        Some((RxToken { frame }, TxToken { tx: &mut self.tx }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if !self.tx.reserve() {
            return None;
        }
        Some(TxToken { tx: &mut self.tx })
    }
}

pub struct RxToken {
    frame: Vec<u8, BUFFER_SIZE>,
}

impl phy::RxToken for RxToken {
    fn consume<T, F>(self, f: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        f(&self.frame)
    }
}

/// Exists only while a Tx buffer is reserved, which it takes when consumed. If it is dropped
/// without being consumed, the buffer stays reserved for the next token.
pub struct TxToken<'a> {
    tx: &'a mut Tx,
}

impl phy::TxToken for TxToken<'_> {
    fn consume<T, F>(self, len: usize, f: F) -> T
    where
        F: FnOnce(&mut [u8]) -> T,
    {
        self.tx.send(len, f)
    }
}
