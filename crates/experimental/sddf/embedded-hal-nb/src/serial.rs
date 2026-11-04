//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;

use sddf::serial::{Consumer, Producer, QueueHandle};
use sddf::sys::serial_client_config_t;

/// Writes notify the driver only when the Tx queue is full. Use [`serial::Write::flush`] to have
/// the driver consume what has been written.
pub struct Client {
    rx: Option<QueueHandle<Consumer>>,
    tx: QueueHandle<Producer>,
}

impl Client {
    /// # Safety
    ///
    /// The queues and data regions described by `config` must be valid and shared only with a
    /// peer which follows the sDDF serial protocol.
    pub unsafe fn new(config: &serial_client_config_t) -> Self {
        let rx_enabled = !config.rx.queue.vaddr.is_null();
        let rx = rx_enabled.then(|| unsafe { QueueHandle::from_connection_resource(&config.rx) });
        let tx = unsafe { QueueHandle::from_connection_resource(&config.tx) };
        Self { rx, tx }
    }

    pub fn rx_channel(&self) -> Option<usize> {
        self.rx.as_ref().map(QueueHandle::channel)
    }

    /// The ID of the channel on which the driver signals this client when it has consumed from
    /// the Tx queue, if a signal has been requested.
    ///
    /// This client never requests such a signal. However, with zeroed shared memory, a signal is
    /// initially considered to have been requested, so one notification may arrive on this
    /// channel.
    pub fn tx_channel(&self) -> usize {
        self.tx.channel()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("receiving is not enabled")]
    NoRx,
}

impl serial::Error for Error {
    fn kind(&self) -> serial::ErrorKind {
        serial::ErrorKind::Other
    }
}

impl serial::ErrorType for Client {
    type Error = Error;
}

impl serial::Read<u8> for Client {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let queue = self.rx.as_mut().ok_or(nb::Error::Other(Error::NoRx))?;
        let c = queue.dequeue().ok_or(nb::Error::WouldBlock)?;
        // The driver may be waiting for space
        queue.signal_producer();
        Ok(c)
    }
}

impl serial::Write<u8> for Client {
    fn write(&mut self, c: u8) -> nb::Result<(), Self::Error> {
        self.tx.enqueue(c).ok_or_else(|| {
            self.tx.notify();
            nb::Error::WouldBlock
        })
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        (self.tx.length_producer() == 0).ok_or_else(|| {
            self.tx.notify();
            nb::Error::WouldBlock
        })
    }
}

impl fmt::Write for Client {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let try_nb = |r| match r {
            Ok(()) => Ok(true),
            Err(nb::Error::WouldBlock) => Ok(false),
            Err(nb::Error::Other(_)) => Err(fmt::Error),
        };
        for b in s.bytes() {
            while !try_nb(serial::Write::write(self, b))? {
                while !try_nb(serial::Write::flush(self))? {
                    core::hint::spin_loop();
                }
                core::hint::spin_loop();
            }
        }
        Ok(())
    }
}
