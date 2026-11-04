//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;

use sddf::resources::DeviceResources;
use sddf::serial::{Consumer, Producer, QueueHandle};
use sddf::sys::serial_driver_config_t;
use sel4_driver_interfaces::HandleInterrupt;
use sel4_microkit::{Channel, ChannelSet, Handler};

pub struct HandlerImpl<Driver> {
    driver: Driver,
    device_channel: Channel,
    rx: Option<QueueHandle<Producer>>,
    tx: QueueHandle<Consumer>,
}

impl<Driver> HandlerImpl<Driver>
where
    Driver: serial::Read<u8> + serial::Write<u8> + HandleInterrupt,
{
    /// The device's IRQ must be the first in `resources`.
    ///
    /// # Safety
    ///
    /// The queues and data regions described by `config` must be valid and shared only with a
    /// peer which follows the sDDF serial protocol.
    pub unsafe fn new(
        driver: Driver,
        resources: &DeviceResources,
        config: &serial_driver_config_t,
    ) -> Self {
        let device_channel = Channel::new(resources.irq(0));
        let rx = config
            .rx_enabled
            .then(|| unsafe { QueueHandle::from_connection_resource(&config.rx) });
        let tx = unsafe { QueueHandle::from_connection_resource(&config.tx) };
        Self {
            driver,
            device_channel,
            rx,
            tx,
        }
    }

    fn rx_return(&mut self) {
        let Some(queue) = &mut self.rx else {
            return;
        };
        let mut enqueued = false;
        // While the queue is full, received characters wait in the device
        while queue
            .try_or_request_signal(|queue| (queue.free() > 0).then_some(()))
            .is_some()
        {
            match self.driver.read() {
                Ok(c) => {
                    assert!(queue.enqueue(c));
                    enqueued = true;
                }
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(err)) => {
                    log::debug!("read error: {err:?}");
                    break;
                }
            }
        }
        if enqueued {
            queue.notify();
        }
    }

    fn tx_provide(&mut self) {
        let queue = &mut self.tx;
        let mut transferred = false;
        while let Some(c) = queue.dequeue() {
            if let Err(err) = nb::block!(self.driver.write(c)) {
                log::debug!("write error: {err:?}");
            }
            transferred = true;
        }
        if transferred {
            queue.signal_producer();
        }
    }
}

impl<Driver> Handler for HandlerImpl<Driver>
where
    Driver: serial::Read<u8> + serial::Write<u8> + HandleInterrupt,
{
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        let mut handled = false;
        if channels.contains(self.device_channel) {
            self.rx_return();
            self.driver.handle_interrupt();
            self.device_channel.irq_ack().unwrap();
            handled = true;
        }
        if let Some(queue) = &self.rx
            && channels.contains(Channel::new(queue.channel()))
        {
            // The client has made space in a full queue
            self.rx_return();
            handled = true;
        }
        if channels.contains(Channel::new(self.tx.channel())) {
            self.tx_provide();
            handled = true;
        }
        assert!(handled, "unexpected channels: {}", channels.display());
        Ok(())
    }
}
