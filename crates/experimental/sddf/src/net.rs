//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Each direction (Rx and Tx) of a connection consists of a pair of queues: a "free" queue of
//! available buffers and an "active" queue of buffers containing data. One side (the [`Filler`])
//! takes buffers from the free queue, fills them, and puts them on the active queue. The other
//! side (the [`Drainer`]) takes buffers from the active queue, consumes their data, and returns
//! them to the free queue. For example, for Rx, the driver is the filler, and for Tx, it is the
//! drainer.

use core::marker::PhantomData;
use core::ops::Range;

use crate::config::impl_config;
use crate::sys;
use crate::utils::{Sealed, int_to_usize, status_to_bool};

pub const BUFFER_SIZE: usize = int_to_usize(sys::NET_BUFFER_SIZE);

impl_config! {
    sys::net_config_check_magic [
        sys::net_driver_config_t,
        sys::net_virt_tx_config_t,
        sys::net_virt_rx_config_t,
        sys::net_copy_config_t,
        sys::net_client_config_t,
        sys::net_vswitch_config_t,
    ]
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct BufferDescriptor(sys::net_buff_desc_t);

impl BufferDescriptor {
    pub fn set_length(&mut self, length: usize) {
        self.0.len = length.try_into().unwrap();
    }

    /// The range of offsets into the data region or of I/O addresses, depending on the connection,
    /// occupied by the buffer's `len` bytes of data.
    pub fn range(&self) -> Range<usize> {
        let len = usize::from(self.0.len);
        assert!(len <= BUFFER_SIZE);
        let start = usize::try_from(self.0.io_or_offset).unwrap();
        start..start.checked_add(len).unwrap()
    }
}

pub trait Role: Sealed {}

pub enum Filler {}

pub enum Drainer {}

impl Sealed for Filler {}
impl Sealed for Drainer {}
impl Role for Filler {}
impl Role for Drainer {}

pub struct QueueHandle<R: Role> {
    inner: sys::net_queue_handle_t,
    _role: PhantomData<R>,
}

impl<R: Role> QueueHandle<R> {
    /// # Safety
    ///
    /// The connection's free and active queues must be valid, have room for `num_buffers` buffer
    /// descriptors, and be shared only with a peer which follows the queue protocol.
    pub unsafe fn from_connection_resource(conn: &sys::net_connection_resource_t) -> Self {
        let mut inner = sys::net_queue_handle_t::default();
        unsafe {
            sys::net_queue_init(
                &mut inner,
                conn.free_queue.vaddr.cast(),
                conn.active_queue.vaddr.cast(),
                conn.num_buffers.into(),
            );
        }
        Self {
            inner,
            _role: PhantomData,
        }
    }

    fn inner(&mut self) -> *mut sys::net_queue_handle_t {
        &mut self.inner
    }

    pub fn as_raw(&self) -> &sys::net_queue_handle_t {
        &self.inner
    }

    pub fn free_len(&self) -> u16 {
        unsafe { sys::net_queue_length(self.inner.free) }
    }

    pub fn active_len(&self) -> u16 {
        unsafe { sys::net_queue_length(self.inner.active) }
    }

    pub fn buffers_init(&mut self, base_addr: usize) {
        unsafe { sys::net_buffers_init(self.inner(), base_addr) }
    }

    pub fn cancel_signal_free(&mut self) {
        unsafe { sys::net_cancel_signal_free(self.inner()) }
    }

    pub fn cancel_signal_active(&mut self) {
        unsafe { sys::net_cancel_signal_active(self.inner()) }
    }
}

impl QueueHandle<Filler> {
    pub fn is_free_empty(&mut self) -> bool {
        unsafe { sys::net_queue_empty_free(self.inner()) }
    }

    pub fn is_active_full(&mut self) -> bool {
        unsafe { sys::net_queue_full_active(self.inner()) }
    }

    pub fn enqueue_active(&mut self, buffer: BufferDescriptor) -> Result<(), BufferDescriptor> {
        status_to_bool(unsafe { sys::net_enqueue_active(self.inner(), buffer.0) })
            .then_some(())
            .ok_or(buffer)
    }

    pub fn dequeue_free(&mut self) -> Option<BufferDescriptor> {
        let mut buffer = BufferDescriptor::default();
        status_to_bool(unsafe { sys::net_dequeue_free(self.inner(), &mut buffer.0) })
            .then_some(buffer)
    }

    pub fn request_signal_free(&mut self) {
        unsafe { sys::net_request_signal_free(self.inner()) }
    }

    pub fn require_signal_active(&mut self) -> bool {
        unsafe { sys::net_require_signal_active(self.inner()) }
    }
}

impl QueueHandle<Drainer> {
    pub fn is_active_empty(&mut self) -> bool {
        unsafe { sys::net_queue_empty_active(self.inner()) }
    }

    pub fn is_free_full(&mut self) -> bool {
        unsafe { sys::net_queue_full_free(self.inner()) }
    }

    pub fn enqueue_free(&mut self, buffer: BufferDescriptor) -> Result<(), BufferDescriptor> {
        status_to_bool(unsafe { sys::net_enqueue_free(self.inner(), buffer.0) })
            .then_some(())
            .ok_or(buffer)
    }

    pub fn dequeue_active(&mut self) -> Option<BufferDescriptor> {
        let mut buffer = BufferDescriptor::default();
        status_to_bool(unsafe { sys::net_dequeue_active(self.inner(), &mut buffer.0) })
            .then_some(buffer)
    }

    pub fn request_signal_active(&mut self) {
        unsafe { sys::net_request_signal_active(self.inner()) }
    }

    pub fn require_signal_free(&mut self) -> bool {
        unsafe { sys::net_require_signal_free(self.inner()) }
    }
}
