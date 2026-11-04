//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Buffer descriptors on the Rx and Tx connections refer to buffers by I/O address. This driver
//! copies frames between the device and those buffers, so it is given the corresponding data
//! regions (and the I/O addresses at which the peer sees them) explicitly.

use core::ops::Range;

use smoltcp::{
    phy::{self, RxToken, TxToken},
    time::Instant,
};

use sddf::net::{BUFFER_SIZE, BufferDescriptor, Drainer, Filler, QueueHandle};
use sddf::resources::{DeviceRegion, DeviceResources};
use sddf::sys::net_driver_config_t;
use sel4_driver_interfaces::HandleInterrupt;
use sel4_microkit::{Channel, ChannelSet, Handler, Infallible};
use sel4_shared_memory::access::ReadOnly;
use sel4_shared_memory::{SharedMemoryPtr, SharedMemoryRef};

pub struct HandlerImpl<Device> {
    dev: Device,
    rx: Rx,
    tx: Tx,
    device_channel: Channel,
}

struct Rx {
    queue: QueueHandle<Filler>,
    data: SharedMemoryRef<'static, [u8]>,
    data_io_addr: DeviceRegion,
    reserved: Option<BufferDescriptor>,
}

struct Tx {
    queue: QueueHandle<Drainer>,
    data: SharedMemoryRef<'static, [u8], ReadOnly>,
    data_io_addr: DeviceRegion,
}

impl Rx {
    fn reserve(&mut self) -> bool {
        if self.reserved.is_none() {
            self.reserved = self.queue.dequeue_free_or_request_signal();
        }
        self.reserved.is_some()
    }

    fn buffer(&mut self, io_range: Range<usize>) -> SharedMemoryPtr<'_, [u8]> {
        let range = self.data_io_addr.io_range_to_offsets(io_range).unwrap();
        self.data.as_mut_ptr().index(range)
    }
}

impl Tx {
    fn buffer(&self, io_range: Range<usize>) -> SharedMemoryPtr<'_, [u8], ReadOnly> {
        let range = self.data_io_addr.io_range_to_offsets(io_range).unwrap();
        self.data.as_ptr().index(range)
    }
}

impl<Device> HandlerImpl<Device> {
    /// Fills the Rx free queue with the buffers in `rx_data`. The device's IRQ must be the first
    /// in `resources`.
    ///
    /// # Safety
    ///
    /// The queues and data regions described by `config`, `rx_data`, and `tx_data` must be valid
    /// and shared only with a peer which follows the sDDF network protocol.
    pub unsafe fn new(
        dev: Device,
        config: &net_driver_config_t,
        rx_data: DeviceRegion,
        tx_data: DeviceRegion,
        resources: &DeviceResources,
    ) -> Self {
        let device_channel = Channel::new(resources.irq(0));
        let mut rx = unsafe {
            Rx {
                queue: QueueHandle::from_connection_resource(&config.virt_rx),
                data: SharedMemoryRef::new(rx_data.region().slice()),
                data_io_addr: rx_data,
                reserved: None,
            }
        };
        let tx = unsafe {
            Tx {
                queue: QueueHandle::from_connection_resource(&config.virt_tx),
                data: SharedMemoryRef::new_read_only(tx_data.region().slice()),
                data_io_addr: tx_data,
            }
        };
        assert!(
            usize::try_from(rx.queue.as_raw().capacity).unwrap() * BUFFER_SIZE
                <= rx_data.region().size()
        );
        rx.queue.buffers_init(rx_data.as_raw().io_addr);
        Self {
            dev,
            rx,
            tx,
            device_channel,
        }
    }
}

impl<Device: phy::Device> HandlerImpl<Device> {
    fn process_rx(&mut self) {
        let mut notify = false;
        while self.rx.reserve()
            && let Some((rx_tok, _tx_tok)) = self.dev.receive(Instant::ZERO)
        {
            let mut buffer = self.rx.reserved.take().unwrap();
            rx_tok.consume(|frame| {
                buffer.set_length(frame.len());
                self.rx.buffer(buffer.range()).copy_from_slice(frame);
            });
            self.rx.queue.enqueue_active(buffer).unwrap();
            notify = true;
        }
        if notify {
            self.rx.queue.signal_active();
        }
    }

    fn process_tx(&mut self) {
        let mut notify = false;
        // Get the token before dequeuing, so that a buffer is never dequeued without a way to send
        // it. Dropping an unused token is harmless.
        while let Some(tx_tok) = self.dev.transmit(Instant::ZERO)
            && let Some(mut buffer) = self.tx.queue.dequeue_active_or_request_signal()
        {
            tx_tok.consume(buffer.range().len(), |frame| {
                self.tx.buffer(buffer.range()).copy_into_slice(frame);
            });
            buffer.set_length(0);
            self.tx.queue.enqueue_free(buffer).unwrap();
            notify = true;
        }
        if notify {
            self.tx.queue.signal_free();
        }
    }
}

impl<Device: phy::Device + HandleInterrupt> Handler for HandlerImpl<Device> {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        // Handle the interrupt first, so that events after this point aren't acknowledged without
        // being processed
        if channels.contains(self.device_channel) {
            self.dev.handle_interrupt();
            self.device_channel.irq_ack().unwrap();
        }
        self.process_rx();
        self.process_tx();
        Ok(())
    }
}
