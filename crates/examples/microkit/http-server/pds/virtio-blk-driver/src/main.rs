//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::pin::Pin;

use virtio_drivers::{
    device::blk::*,
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use sddf::blk::{QueueHandle, Server, StorageInfo, TRANSFER_SIZE};
use sddf::resources::DeviceRegion;
use sddf::sys::blk_req_code::{BLK_REQ_BARRIER, BLK_REQ_FLUSH, BLK_REQ_READ, BLK_REQ_WRITE};
use sddf::sys::blk_resp_status::{BLK_RESP_ERR_INVALID_PARAM, BLK_RESP_ERR_IO, BLK_RESP_OK};
use sddf::sys::{
    blk_driver_config_t, blk_req_t, blk_resp_status_t, blk_resp_t, blk_storage_info_t,
};
use sel4_microkit::{Channel, ChannelSet, Handler, Infallible, protection_domain};
use sel4_shared_memory::SharedMemoryRef;
use sel4_virtio_hal_impl::HalImpl;

// HACK hard-coded in virtio-drivers
const QUEUE_SIZE: usize = 4;

const SECTORS_PER_TRANSFER: usize = TRANSFER_SIZE / SECTOR_SIZE;

const MMIO_REGION: usize = 0;
const DMA_REGION: usize = 1;
const DATA_REGION: usize = 2;

#[protection_domain(
    heap_size = 64 * 1024,
)]
fn init() -> HandlerImpl {
    let resources = sddf::device_resources!();
    let config = sddf::config!(".blk_driver_config", blk_driver_config_t);

    let dma = resources.region(DMA_REGION);
    HalImpl::init(
        dma.region().size(),
        dma.region().vaddr().as_ptr() as usize,
        dma.as_raw().io_addr,
    );

    let mut dev = {
        let mmio = resources.region(MMIO_REGION);
        let header = mmio.region().vaddr().cast::<VirtIOHeader>();
        let transport = unsafe { MmioTransport::new(header, mmio.region().size()) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Block);
        VirtIOBlk::<HalImpl, MmioTransport>::new(transport).unwrap()
    };

    let data = *resources.region(DATA_REGION);
    let data_region = unsafe { SharedMemoryRef::new(data.region().slice()) };

    let queue = unsafe { QueueHandle::from_connection_resource(&config.virt) };

    let mut storage_info = unsafe { StorageInfo::<Server>::from_connection_resource(&config.virt) };
    storage_info.publish(blk_storage_info_t {
        read_only: dev.readonly(),
        sector_size: SECTOR_SIZE.try_into().unwrap(),
        block_size: 1,
        queue_depth: QUEUE_SIZE.try_into().unwrap(),
        capacity: dev.capacity() / u64::try_from(SECTORS_PER_TRANSFER).unwrap(),
        ..Default::default()
    });

    let device_channel = Channel::new(resources.irq(0));

    dev.ack_interrupt();
    device_channel.irq_ack().unwrap();

    HandlerImpl {
        dev,
        data_region,
        data_io_addr: data,
        queue,
        device_channel,
        pending: BTreeMap::new(),
    }
}

struct HandlerImpl {
    dev: VirtIOBlk<HalImpl, MmioTransport<'static>>,
    data_region: SharedMemoryRef<'static, [u8]>,
    data_io_addr: DeviceRegion,
    queue: QueueHandle<Server>,
    device_channel: Channel,
    pending: BTreeMap<u16, Pin<Box<PendingEntry>>>,
}

struct PendingEntry {
    client_req: blk_req_t,
    virtio_req: BlkReq,
    virtio_resp: BlkResp,
}

impl HandlerImpl {
    fn buffer(&mut self, req: &blk_req_t) -> Option<&'static mut [u8]> {
        let len = usize::from(req.count).checked_mul(TRANSFER_SIZE)?;
        let start = usize::try_from(req.io_or_offset).ok()?;
        let range = self
            .data_io_addr
            .io_range_to_offsets(start..start.checked_add(len)?)?;
        let mut ptr = self.data_region.as_mut_ptr().index(range).as_raw_ptr();
        // The client does not access this buffer until we respond.
        Some(unsafe { ptr.as_mut() })
    }

    fn respond(&mut self, req: &blk_req_t, status: blk_resp_status_t) {
        let success_count = if status == BLK_RESP_OK { req.count } else { 0 };
        self.queue
            .enqueue_resp(blk_resp_t {
                status,
                success_count,
                id: req.id,
            })
            .unwrap();
    }

    fn complete_pending(&mut self) -> bool {
        let mut progress = false;
        while let Some(token) = self.dev.peek_used() {
            let mut pending_entry = self.pending.remove(&token).unwrap();
            let client_req = pending_entry.client_req;
            let buf = self.buffer(&client_req).unwrap();
            let pending_entry = &mut *pending_entry;
            let result = unsafe {
                match client_req.code {
                    BLK_REQ_READ => self.dev.complete_read_blocks(
                        token,
                        &pending_entry.virtio_req,
                        buf,
                        &mut pending_entry.virtio_resp,
                    ),
                    BLK_REQ_WRITE => self.dev.complete_write_blocks(
                        token,
                        &pending_entry.virtio_req,
                        buf,
                        &mut pending_entry.virtio_resp,
                    ),
                    _ => unreachable!(),
                }
            };
            let status = match (result, pending_entry.virtio_resp.status()) {
                (Ok(()), RespStatus::OK) => BLK_RESP_OK,
                _ => BLK_RESP_ERR_IO,
            };
            self.respond(&client_req, status);
            progress = true;
        }
        progress
    }

    fn issue_requests(&mut self) -> bool {
        let mut progress = false;
        while self.pending.len() < QUEUE_SIZE
            && !self.queue.is_req_empty()
            && !self.queue.is_resp_full()
        {
            let client_req = self.queue.dequeue_req().unwrap();
            progress = true;
            let code = client_req.code;
            match code {
                BLK_REQ_READ | BLK_REQ_WRITE => {}
                BLK_REQ_FLUSH | BLK_REQ_BARRIER => {
                    self.respond(&client_req, BLK_RESP_OK);
                    continue;
                }
                _ => {
                    log::warn!("invalid request code: {code}");
                    self.respond(&client_req, BLK_RESP_ERR_INVALID_PARAM);
                    continue;
                }
            }
            let block_id = usize::try_from(client_req.block_number)
                .ok()
                .and_then(|n| n.checked_mul(SECTORS_PER_TRANSFER));
            let (Some(block_id), Some(buf)) = (block_id, self.buffer(&client_req)) else {
                self.respond(&client_req, BLK_RESP_ERR_INVALID_PARAM);
                continue;
            };
            if client_req.count == 0 || (code == BLK_REQ_WRITE && self.dev.readonly()) {
                self.respond(&client_req, BLK_RESP_ERR_INVALID_PARAM);
                continue;
            }
            let mut pending_entry = Box::pin(PendingEntry {
                client_req,
                virtio_req: BlkReq::default(),
                virtio_resp: BlkResp::default(),
            });
            let result = unsafe {
                let pending_entry = &mut *pending_entry;
                match code {
                    BLK_REQ_READ => self.dev.read_blocks_nb(
                        block_id,
                        &mut pending_entry.virtio_req,
                        buf,
                        &mut pending_entry.virtio_resp,
                    ),
                    BLK_REQ_WRITE => self.dev.write_blocks_nb(
                        block_id,
                        &mut pending_entry.virtio_req,
                        buf,
                        &mut pending_entry.virtio_resp,
                    ),
                    _ => unreachable!(),
                }
            };
            match result {
                Ok(token) => {
                    assert!(self.pending.insert(token, pending_entry).is_none());
                }
                Err(_) => {
                    self.respond(&client_req, BLK_RESP_ERR_IO);
                }
            }
        }
        progress
    }
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        let mut notify = false;
        notify |= self.complete_pending();
        notify |= self.issue_requests();
        if notify {
            self.queue.notify();
        }
        if channels.contains(self.device_channel) {
            self.dev.ack_interrupt();
            self.device_channel.irq_ack().unwrap();
        }
        Ok(())
    }
}
