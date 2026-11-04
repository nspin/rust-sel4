//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::num::NonZero;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use sddf::serial::{Consumer, Producer, QueueHandle};
use sddf::sys::serial_client_config_t;

/// Writes notify the driver only when the Tx queue is full. Use
/// [`poll_flush`](sel4_async_io::Write::poll_flush) to have the driver consume what has been
/// written.
///
/// The owner of the client must call [`wake`](Self::wake) whenever it is notified on
/// [`rx_channel`](Self::rx_channel) or [`tx_channel`](Self::tx_channel). A notification on either
/// channel is always fine to receive, even if nothing is waiting for it.
pub struct Client {
    rx: Option<QueueHandle<Consumer>>,
    tx: QueueHandle<Producer>,
    rx_waker: Option<Waker>,
    tx_waker: Option<Waker>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("receiving is not enabled")]
    NoRx,
}

impl sel4_async_io::Error for Error {
    fn kind(&self) -> sel4_async_io::ErrorKind {
        sel4_async_io::ErrorKind::Unsupported
    }
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
        Self {
            rx,
            tx,
            rx_waker: None,
            tx_waker: None,
        }
    }

    pub fn rx_channel(&self) -> Option<usize> {
        self.rx.as_ref().map(QueueHandle::channel)
    }

    /// The ID of the channel on which the driver signals this client when it has consumed from
    /// the Tx queue, if a signal has been requested.
    ///
    /// With zeroed shared memory, a signal is initially considered to have been requested, so a
    /// notification may arrive on this channel even if this client has not requested one.
    pub fn tx_channel(&self) -> usize {
        self.tx.channel()
    }

    pub fn wake(&mut self) {
        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }
        if let Some(waker) = self.tx_waker.take() {
            waker.wake();
        }
    }
}

impl sel4_async_io::ErrorType for Client {
    type Error = Error;
}

impl sel4_async_io::Read for Client {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        let this = self.get_mut();
        let Some(queue) = &mut this.rx else {
            return Poll::Ready(Err(Error::NoRx));
        };
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        let mut n = 0;
        while n < buf.len()
            && let Some(c) = queue.dequeue()
        {
            buf[n] = c;
            n += 1;
        }
        if n == 0 {
            // The driver always notifies after enqueuing, so no signal needs to be requested
            this.rx_waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
        // The driver may be waiting for space
        queue.signal_producer();
        Poll::Ready(Ok(n))
    }
}

impl sel4_async_io::Write for Client {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let this = self.get_mut();
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }
        match this
            .tx
            .try_or_request_signal(|tx| NonZero::new(tx.enqueue_batch(buf)))
        {
            Some(n) => Poll::Ready(Ok(n.get())),
            None => {
                this.tx_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    /// Completes once the driver has consumed everything from the Tx queue (which does not
    /// necessarily mean that it has been transmitted).
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let this = self.get_mut();
        match this
            .tx
            .try_or_request_signal(|tx| (tx.length_producer() == 0).then_some(()))
        {
            Some(()) => Poll::Ready(Ok(())),
            None => {
                this.tx_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
