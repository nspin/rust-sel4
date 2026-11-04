//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Block numbers and counts are in units of [`TRANSFER_SIZE`] bytes.

use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use crate::config::impl_config;
use crate::sys;
use crate::utils::{Sealed, int_to_usize, status_to_bool};

pub const TRANSFER_SIZE: usize = int_to_usize(sys::BLK_TRANSFER_SIZE);

pub const STORAGE_INFO_REGION_SIZE: usize = int_to_usize(sys::BLK_STORAGE_INFO_REGION_SIZE);

impl_config! {
    sys::blk_config_check_magic [
        sys::blk_driver_config_t,
        sys::blk_virt_config_t,
        sys::blk_client_config_t,
    ]
}

pub trait Role: Sealed {}

pub enum Client {}

pub enum Server {}

impl Sealed for Client {}
impl Sealed for Server {}
impl Role for Client {}
impl Role for Server {}

pub struct QueueHandle<R: Role> {
    inner: sys::blk_queue_handle_t,
    channel: sys::sddf_channel,
    _role: PhantomData<R>,
}

impl<R: Role> QueueHandle<R> {
    /// # Safety
    ///
    /// The connection's request and response queues must be valid, have room for `num_buffers`
    /// entries, and be shared only with a peer which follows the queue protocol.
    pub unsafe fn from_connection_resource(conn: &sys::blk_connection_resource_t) -> Self {
        let mut inner = sys::blk_queue_handle_t::default();
        unsafe {
            sys::blk_queue_init(
                &mut inner,
                conn.req_queue.vaddr.cast(),
                conn.resp_queue.vaddr.cast(),
                conn.num_buffers.into(),
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

    fn inner(&mut self) -> *mut sys::blk_queue_handle_t {
        &mut self.inner
    }

    pub fn as_raw(&self) -> &sys::blk_queue_handle_t {
        &self.inner
    }

    pub fn req_len(&mut self) -> u32 {
        unsafe { sys::blk_queue_length_req(self.inner()) }
    }

    pub fn resp_len(&mut self) -> u32 {
        unsafe { sys::blk_queue_length_resp(self.inner()) }
    }
}

impl QueueHandle<Client> {
    pub fn is_req_full(&mut self) -> bool {
        unsafe { sys::blk_queue_full_req(self.inner()) }
    }

    pub fn is_resp_empty(&mut self) -> bool {
        unsafe { sys::blk_queue_empty_resp(self.inner()) }
    }

    pub fn enqueue_req(&mut self, req: sys::blk_req_t) -> Result<(), sys::blk_req_t> {
        let ret = unsafe {
            sys::blk_enqueue_req(
                self.inner(),
                req.code,
                req.io_or_offset.try_into().unwrap(),
                req.block_number,
                req.count,
                req.id,
            )
        };
        status_to_bool(ret).then_some(()).ok_or(req)
    }

    pub fn dequeue_resp(&mut self) -> Option<sys::blk_resp_t> {
        let mut resp = sys::blk_resp_t::default();
        let ret = unsafe {
            sys::blk_dequeue_resp(
                self.inner(),
                &mut resp.status,
                &mut resp.success_count,
                &mut resp.id,
            )
        };
        status_to_bool(ret).then_some(resp)
    }

    pub fn plug_req(&mut self) {
        unsafe { sys::blk_queue_plug_req(self.inner()) }
    }

    pub fn unplug_req(&mut self) {
        unsafe { sys::blk_queue_unplug_req(self.inner()) }
    }
}

impl QueueHandle<Server> {
    pub fn is_req_empty(&mut self) -> bool {
        unsafe { sys::blk_queue_empty_req(self.inner()) }
    }

    pub fn is_resp_full(&mut self) -> bool {
        unsafe { sys::blk_queue_full_resp(self.inner()) }
    }

    pub fn dequeue_req(&mut self) -> Option<sys::blk_req_t> {
        let mut req = sys::blk_req_t::default();
        let mut io_or_offset = 0;
        let ret = unsafe {
            sys::blk_dequeue_req(
                self.inner(),
                &mut req.code,
                &mut io_or_offset,
                &mut req.block_number,
                &mut req.count,
                &mut req.id,
            )
        };
        req.io_or_offset = io_or_offset.try_into().unwrap();
        status_to_bool(ret).then_some(req)
    }

    pub fn enqueue_resp(&mut self, resp: sys::blk_resp_t) -> Result<(), sys::blk_resp_t> {
        let ret = unsafe {
            sys::blk_enqueue_resp(self.inner(), resp.status, resp.success_count, resp.id)
        };
        status_to_bool(ret).then_some(()).ok_or(resp)
    }

    pub fn is_req_plugged(&mut self) -> bool {
        unsafe { sys::blk_queue_plugged_req(self.inner()) }
    }
}

/// # Safety
///
/// The connection's storage information region must contain a valid `blk_storage_info_t`, shared
/// only with a peer which follows the protocol.
pub unsafe fn is_storage_info_ready(config: &sys::blk_client_config_t) -> bool {
    unsafe { StorageInfo::<Client>::from_connection_resource(&config.virt) }.is_ready()
}

pub struct StorageInfo<R: Role> {
    ptr: NonNull<sys::blk_storage_info_t>,
    _role: PhantomData<R>,
}

impl<R: Role> StorageInfo<R> {
    /// # Safety
    ///
    /// The connection's storage information region must contain a valid `blk_storage_info_t`,
    /// shared only with a peer which follows the protocol.
    pub unsafe fn from_connection_resource(conn: &sys::blk_connection_resource_t) -> Self {
        Self {
            ptr: NonNull::new(conn.storage_info.vaddr.cast()).unwrap(),
            _role: PhantomData,
        }
    }
}

impl StorageInfo<Client> {
    pub fn is_ready(&self) -> bool {
        unsafe { sys::blk_storage_is_ready(self.ptr.as_ptr()) }
    }

    pub fn read(&self) -> Option<sys::blk_storage_info_t> {
        // The acquire in `is_ready` orders this read after the server's writes. The information is
        // copied out in one read, so that it is never fetched twice.
        self.is_ready()
            .then(|| unsafe { ptr::read_volatile(self.ptr.as_ptr()) })
    }
}

impl StorageInfo<Server> {
    pub fn publish(&mut self, mut info: sys::blk_storage_info_t) {
        info.ready = false;
        unsafe {
            ptr::write_volatile(self.ptr.as_ptr(), info);
            sys::blk_storage_set_ready(self.ptr.as_ptr(), true);
        }
    }
}
