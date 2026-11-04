//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Each queue carries bytes in one direction, from a [`Producer`] to a [`Consumer`]. Note that,
//! following sDDF, the producer calls [`QueueHandle::request_consumer_signal`] to ask the consumer
//! to signal it, and the consumer calls [`QueueHandle::require_consumer_signal`] to check whether
//! it must.

use core::marker::PhantomData;

use crate::config::impl_config;
use crate::sys;
use crate::utils::{Sealed, int_to_usize, status_to_bool};

impl_config! {
    sys::serial_config_check_magic [
        sys::serial_driver_config_t,
        sys::serial_virt_rx_config_t,
        sys::serial_virt_tx_config_t,
        sys::serial_client_config_t,
    ]
}

pub trait Role: Sealed {}

pub enum Producer {}

pub enum Consumer {}

impl Sealed for Producer {}
impl Sealed for Consumer {}
impl Role for Producer {}
impl Role for Consumer {}

pub struct QueueHandle<R: Role> {
    inner: sys::serial_queue_handle_t,
    channel: sys::sddf_channel,
    _role: PhantomData<R>,
}

impl<R: Role> QueueHandle<R> {
    /// # Safety
    ///
    /// The connection's queue and data regions must be valid, and shared only with a peer which
    /// follows the queue protocol.
    pub unsafe fn from_connection_resource(conn: &sys::serial_connection_resource_t) -> Self {
        let mut inner = sys::serial_queue_handle_t::default();
        unsafe {
            sys::serial_queue_init(
                &mut inner,
                conn.queue.vaddr.cast(),
                conn.data.size.try_into().unwrap(),
                conn.data.vaddr.cast(),
            );
        }
        Self {
            inner,
            channel: conn.id.into(),
            _role: PhantomData,
        }
    }

    pub fn notify(&self) {
        unsafe { sys::sddf_notify(self.channel) }
    }

    pub fn channel(&self) -> usize {
        int_to_usize(self.channel)
    }

    fn inner(&mut self) -> *mut sys::serial_queue_handle_t {
        &mut self.inner
    }

    pub fn as_raw(&self) -> &sys::serial_queue_handle_t {
        &self.inner
    }

    pub fn cancel_consumer_signal(&mut self) {
        unsafe { sys::serial_cancel_consumer_signal(self.inner()) }
    }
}

impl QueueHandle<Producer> {
    pub fn length_producer(&mut self) -> u32 {
        unsafe { sys::serial_queue_length_producer(self.inner()) }
    }

    pub fn free(&mut self) -> u32 {
        unsafe { sys::serial_queue_free(self.inner()) }
    }

    pub fn enqueue(&mut self, c: u8) -> bool {
        status_to_bool(unsafe { sys::serial_enqueue(self.inner(), c as _) })
    }

    pub fn enqueue_batch(&mut self, src: &[u8]) -> usize {
        int_to_usize(unsafe {
            sys::serial_enqueue_batch(
                self.inner(),
                src.len().try_into().unwrap(),
                src.as_ptr().cast(),
            )
        })
    }

    pub fn request_consumer_signal(&mut self) {
        unsafe { sys::serial_request_consumer_signal(self.inner()) }
    }

    /// Tries `f`. If it fails, notifies the consumer so that it consumes from the queue, requests
    /// a signal for when it has, and tries `f` again, canceling the request if that succeeds. If
    /// both tries fail, returns `None` with the signal still requested.
    pub fn try_or_request_signal<T>(
        &mut self,
        mut f: impl FnMut(&mut Self) -> Option<T>,
    ) -> Option<T> {
        f(self).or_else(|| {
            self.notify();
            self.request_consumer_signal();
            let r = f(self)?;
            self.cancel_consumer_signal();
            Some(r)
        })
    }
}

impl QueueHandle<Consumer> {
    pub fn signal_producer(&mut self) {
        if self.require_consumer_signal() {
            self.cancel_consumer_signal();
            self.notify();
        }
    }

    pub fn length_consumer(&mut self) -> u32 {
        unsafe { sys::serial_queue_length_consumer(self.inner()) }
    }

    pub fn dequeue(&mut self) -> Option<u8> {
        let mut c = 0;
        status_to_bool(unsafe { sys::serial_dequeue(self.inner(), &mut c) }).then_some(c as _)
    }

    pub fn require_consumer_signal(&mut self) -> bool {
        unsafe { sys::serial_require_consumer_signal(self.inner()) }
    }
}
