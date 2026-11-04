//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![allow(clippy::useless_conversion)]

use core::cell::RefCell;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem;
use core::num::NonZero;
use core::ops::Range;
use core::task::{Poll, Waker};

use sddf::blk::{Client, QueueHandle, StorageInfo, TRANSFER_SIZE};
use sddf::resources::Region;
use sddf::sys::blk_req_code::{BLK_REQ_BARRIER, BLK_REQ_FLUSH, BLK_REQ_READ, BLK_REQ_WRITE};
use sddf::sys::blk_resp_status::BLK_RESP_OK;
use sddf::sys::{blk_client_config_t, blk_req_t, blk_resp_status_t, blk_resp_t};
use sel4_async_block_io::{BlockIO, BlockIOLayout, Operation, RuntimeBlockSize, access::Access};
use sel4_shared_memory::SharedMemoryRef;

mod bookkeeping;

use bookkeeping::RequestTable;

const DEFAULT_MAX_OUTSTANDING_REQUESTS: usize = 32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, thiserror::Error)]
#[error("block request failed with status {0}")]
pub struct Error(blk_resp_status_t);

impl Error {
    pub fn status(&self) -> blk_resp_status_t {
        self.0
    }

    pub fn check_status(status: blk_resp_status_t) -> Result<(), Self> {
        if status == BLK_RESP_OK {
            Ok(())
        } else {
            Err(Self(status))
        }
    }
}

/// At most `min(N, queue capacity, data region size / block size)` requests can be outstanding at
/// once. Further requests wait.
///
/// Like other sDDF clients, this client trusts its virtualiser, and panics if a response does not
/// match an outstanding request.
pub struct SddfBlockIO<P, const N: usize = DEFAULT_MAX_OUTSTANDING_REQUESTS> {
    inner: RefCell<Inner>,
    requests: RequestTable<RequestState, N>,
    transfer_units_per_block: u16,
    transfer_units_per_request: usize,
    num_transfer_units: u64,
    _phantom: PhantomData<P>,
}

struct Inner {
    data_region: SharedMemoryRef<'static, [u8]>,
    queue: QueueHandle<Client>,
}

enum RequestState {
    Pending { waker: Option<Waker> },
    Canceled,
    Complete { result: Result<(), Error> },
}

enum Request<'a> {
    Flush,
    Barrier,
    Read {
        start_transfer_idx: u64,
        buf: &'a mut [u8],
    },
    Write {
        start_transfer_idx: u64,
        buf: &'a [u8],
    },
}

impl Request<'_> {
    fn pack_request(&self, transfer_units_per_request: usize, request_index: usize) -> blk_req_t {
        let (code, block_number, len) = match self {
            Request::Flush => (BLK_REQ_FLUSH, 0, 0),
            Request::Barrier => (BLK_REQ_BARRIER, 0, 0),
            Request::Read {
                start_transfer_idx,
                buf,
            } => (BLK_REQ_READ, *start_transfer_idx, buf.len()),
            Request::Write {
                start_transfer_idx,
                buf,
            } => (BLK_REQ_WRITE, *start_transfer_idx, buf.len()),
        };
        blk_req_t {
            code,
            io_or_offset: (request_index * transfer_units_per_request * TRANSFER_SIZE)
                .try_into()
                .unwrap(),
            block_number,
            count: (len / TRANSFER_SIZE).try_into().unwrap(),
            id: request_index.try_into().unwrap(),
        }
    }
}

struct CancelOnDrop<'a, P: Access, const N: usize> {
    io: &'a SddfBlockIO<P, N>,
    request_index: usize,
}

impl<P: Access, const N: usize> Drop for CancelOnDrop<'_, P, N> {
    fn drop(&mut self) {
        self.io.cancel_request(self.request_index);
    }
}

impl<P: Access, const N: usize> SddfBlockIO<P, N> {
    /// The block size is the storage information's optimal block size (in units of
    /// [`TRANSFER_SIZE`]). A block size of 0 is treated as 1.
    ///
    /// Panics if the storage information is not yet ready (see
    /// [`sddf::blk::is_storage_info_ready`]), or if the data region is smaller than a block.
    ///
    /// # Safety
    ///
    /// The queues and regions described by `config` must be valid and shared only with a peer
    /// which follows the sDDF block protocol.
    pub unsafe fn new(config: &blk_client_config_t) -> Self {
        let storage_info = unsafe { StorageInfo::<Client>::from_connection_resource(&config.virt) }
            .read()
            .expect("storage information not ready");

        let data_region =
            unsafe { SharedMemoryRef::new(Region::from_resource(&config.data).unwrap().slice()) };

        let queue = unsafe { QueueHandle::from_connection_resource(&config.virt) };

        let transfer_units_per_block = storage_info.block_size.max(1);

        let region_num_blocks =
            data_region.as_ptr().len() / (TRANSFER_SIZE * usize::from(transfer_units_per_block));
        let num_slots = usize::try_from(queue.as_raw().capacity)
            .unwrap()
            .min(N)
            .min(region_num_blocks);
        assert!(num_slots > 0, "data region too small");
        let blocks_per_request = region_num_blocks / num_slots;
        let transfer_units_per_request = blocks_per_request * usize::from(transfer_units_per_block);

        Self {
            inner: RefCell::new(Inner { data_region, queue }),
            requests: RequestTable::new(num_slots),
            transfer_units_per_block,
            transfer_units_per_request,
            num_transfer_units: storage_info.capacity,
            _phantom: PhantomData,
        }
    }

    /// Issues a barrier request, which asks the driver to complete all requests issued before it
    /// before starting any issued after it. Completes when the driver responds.
    ///
    /// Support for barriers varies between sDDF drivers. Some reject them, in which case this
    /// returns an error, and others accept them without enforcing any ordering.
    pub async fn barrier(&self) -> Result<(), Error> {
        self.issue_and_await(Request::Barrier).await
    }

    async fn issue_and_await(&self, mut request: Request<'_>) -> Result<(), Error> {
        let request_index = self
            .requests
            .insert(RequestState::Pending { waker: None })
            .await;
        self.issue_request(request_index, &request);
        let cancel_on_drop = CancelOnDrop {
            io: self,
            request_index,
        };
        let result = poll_fn(|cx| self.poll_request(request_index, &mut request, cx.waker())).await;
        mem::forget(cancel_on_drop);
        result
    }

    /// The length of a read or write request's buffer must be a multiple of [`TRANSFER_SIZE`] and
    /// at most `transfer_units_per_request` units of [`TRANSFER_SIZE`].
    fn issue_request(&self, request_index: usize, request: &Request<'_>) {
        let req = request.pack_request(self.transfer_units_per_request, request_index);

        let mut inner = self.inner.borrow_mut();

        if let Request::Write { buf, .. } = request {
            inner
                .data_region
                .as_mut_ptr()
                .index(self.slot_range(request_index, buf.len()))
                .copy_from_slice(buf);
        }

        inner.queue.enqueue_req(req).unwrap();
        inner.queue.notify();
    }

    fn cancel_request(&self, request_index: usize) {
        self.requests
            .update(request_index, |state| match state {
                RequestState::Pending { .. } => (Some(RequestState::Canceled), ()),
                RequestState::Complete { .. } => (None, ()),
                RequestState::Canceled => panic!("request already canceled"),
            })
            .unwrap()
    }

    fn poll_request(
        &self,
        request_index: usize,
        request: &mut Request<'_>,
        waker: &Waker,
    ) -> Poll<Result<(), Error>> {
        self.requests
            .update(request_index, |state| match state {
                RequestState::Pending { .. } => (
                    Some(RequestState::Pending {
                        waker: Some(waker.clone()),
                    }),
                    Poll::Pending,
                ),
                RequestState::Complete { result } => {
                    if result.is_ok()
                        && let Request::Read { buf, .. } = request
                    {
                        self.inner
                            .borrow()
                            .data_region
                            .as_ptr()
                            .index(self.slot_range(request_index, buf.len()))
                            .copy_into_slice(buf);
                    }
                    (None, Poll::Ready(result))
                }
                RequestState::Canceled => panic!("request canceled"),
            })
            .unwrap()
    }

    pub fn poll(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        let mut progress = false;
        while let Some(resp) = inner.queue.dequeue_resp() {
            self.handle_response(resp);
            progress = true;
        }
        progress
    }

    fn handle_response(&self, resp: blk_resp_t) {
        let request_index = usize::try_from(resp.id).unwrap();

        let result = Error::check_status(resp.status);

        self.requests
            .update(request_index, |state| match state {
                RequestState::Pending { waker } => {
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    (Some(RequestState::Complete { result }), ())
                }
                RequestState::Canceled => (None, ()),
                RequestState::Complete { .. } => {
                    panic!("duplicate response for request {request_index}")
                }
            })
            .unwrap_or_else(|err| panic!("response for request {request_index}: {err:?}"))
    }

    fn slot_range(&self, request_index: usize, len: usize) -> Range<usize> {
        let start = request_index * self.transfer_units_per_request * TRANSFER_SIZE;
        start..start + len
    }
}

impl<P: Access, const N: usize> BlockIOLayout for SddfBlockIO<P, N> {
    type Error = Error;

    type BlockSize = RuntimeBlockSize;

    fn block_size(&self) -> Self::BlockSize {
        let bytes = TRANSFER_SIZE * usize::from(self.transfer_units_per_block);
        RuntimeBlockSize::new(NonZero::new(bytes).unwrap())
    }

    fn num_blocks(&self) -> u64 {
        self.num_transfer_units / u64::from(self.transfer_units_per_block)
    }
}

impl<P: Access, const N: usize> BlockIO<P> for SddfBlockIO<P, N> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        mut operation: Operation<'_, P>,
    ) -> Result<(), Self::Error> {
        let first_transfer_idx = start_block_idx * u64::from(self.transfer_units_per_block);
        for (i, chunk) in operation
            .chunks(self.transfer_units_per_request * TRANSFER_SIZE)
            .enumerate()
        {
            let start_transfer_idx =
                first_transfer_idx + u64::try_from(i * self.transfer_units_per_request).unwrap();
            self.issue_and_await(match chunk {
                Operation::Read { buf, .. } => Request::Read {
                    start_transfer_idx,
                    buf,
                },
                Operation::Write { buf, .. } => Request::Write {
                    start_transfer_idx,
                    buf,
                },
            })
            .await?;
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), Self::Error> {
        self.issue_and_await(Request::Flush).await
    }
}
