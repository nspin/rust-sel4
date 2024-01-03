//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(async_fn_in_trait)]
#![feature(cfg_target_thread_local)]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(pattern)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![feature(thread_local)]
#![feature(try_blocks)]

extern crate alloc;

use alloc::rc::Rc;
use core::time::Duration;

use smoltcp::iface::Config;
use smoltcp::phy::{Device, Medium};
use smoltcp::wire::{EthernetAddress, HardwareAddress};

use sel4_async_block_io::{
    constant_block_sizes::BlockSize512, disk::Disk, CachedBlockIO, ConstantBlockSize,
};
use sel4_async_time::Instant;
use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_externally_shared::{ExternallySharedRef, ExternallySharedRefExt};
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_microkit::{memory_region_symbol, protection_domain, Handler};
use sel4_shared_ring_buffer::RingBuffers;
use sel4_shared_ring_buffer_block_io::SharedRingBufferBlockIO;
use sel4_shared_ring_buffer_smoltcp::DeviceImpl;

use microkit_http_server_example_server_core::run_server;

mod block_client;
mod config;
mod handler;
mod net_client;
mod timer_client;

use block_client::BlockClient;
use config::channels;
use handler::HandlerImpl;
use net_client::NetClient;
use timer_client::TimerClient;

const BLOCK_CACHE_SIZE_IN_BLOCKS: usize = 128;

const MAX_NUM_SIMULTANEOUS_CONNECTIONS: usize = 32;

const CERT_PEM: &str = concat!(include_str!(concat!(env!("OUT_DIR"), "/cert.pem")), "\0");
const PRIV_PEM: &str = concat!(include_str!(concat!(env!("OUT_DIR"), "/priv.pem")), "\0");

const LOG_LEVEL: LevelFilter = {
    // LevelFilter::Trace
    // LevelFilter::Debug
    LevelFilter::Info
    // LevelFilter::Warn
};

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| !meta.target().starts_with("sel4_sys"))
    .write(|s| sel4::debug_print!("{}", s))
    .build();

#[protection_domain(
    heap_size = 16 * 1024 * 1024,
)]
fn init() -> impl Handler {
    LOGGER.set().unwrap();

    setup_newlib();

    let timer_client = TimerClient::new(channels::TIMER_DRIVER);
    let net_client = NetClient::new(channels::NET_DRIVER);
    let block_client = BlockClient::new(channels::BLOCK_DRIVER);

    let timer_client = Rc::new(timer_client);

    let notify_net: fn() = || channels::NET_DRIVER.notify();
    let notify_block: fn() = || channels::BLOCK_DRIVER.notify();

    let net_device = {
        let dma_region = unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(virtio_net_client_dma_vaddr: *mut [u8], n = config::VIRTIO_NET_CLIENT_DMA_SIZE),
            )
        };

        let bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), 1);

        DeviceImpl::new(
            dma_region,
            bounce_buffer_allocator,
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_rx_free: *mut _))
                },
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_rx_used: *mut _))
                },
                notify_net,
            ),
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_tx_free: *mut _))
                },
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_tx_used: *mut _))
                },
                notify_net,
            ),
            16,
            2048,
            1500,
        )
        .unwrap()
    };

    let net_config = {
        assert_eq!(net_device.capabilities().medium, Medium::Ethernet);
        let mac_address = EthernetAddress(net_client.get_mac_address().0);
        let hardware_addr = HardwareAddress::Ethernet(mac_address);
        let mut this = Config::new(hardware_addr);
        this.random_seed = 0;
        this
    };

    let num_blocks = block_client.get_num_blocks();

    let shared_block_io = {
        let dma_region = unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(virtio_blk_client_dma_vaddr: *mut [u8], n = config::VIRTIO_BLK_CLIENT_DMA_SIZE),
            )
        };

        let bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), 1);

        SharedRingBufferBlockIO::new(
            BlockSize512::BLOCK_SIZE,
            num_blocks,
            dma_region,
            bounce_buffer_allocator,
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_blk_free: *mut _)) },
                unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_blk_used: *mut _)) },
                notify_block,
            ),
        )
    };

    let now_fn = {
        let timer_client = timer_client.clone();
        move || Instant::ZERO + Duration::from_micros(timer_client.now())
    };

    HandlerImpl::new(
        channels::TIMER_DRIVER,
        channels::NET_DRIVER,
        channels::BLOCK_DRIVER,
        timer_client,
        net_device,
        net_config,
        shared_block_io.clone(),
        |timers_ctx, network_ctx, spawner| async move {
            let fs_block_io = shared_block_io.clone();
            let fs_block_io = CachedBlockIO::new(fs_block_io.clone(), BLOCK_CACHE_SIZE_IN_BLOCKS);
            let disk = Disk::new(fs_block_io);
            let entry = disk.read_mbr().await.unwrap().partition(0).unwrap();
            let fs_block_io = disk.partition_using_mbr(&entry);
            let fs_block_io = Rc::new(fs_block_io);
            run_server(
                now_fn,
                timers_ctx,
                network_ctx,
                fs_block_io,
                spawner,
                CERT_PEM,
                PRIV_PEM,
                MAX_NUM_SIMULTANEOUS_CONNECTIONS,
            )
            .await
        },
    )
}

fn setup_newlib() {
    use sel4_newlib::*;

    set_static_heap_for_sbrk({
        static HEAP: StaticHeap<{ 1024 * 1024 }> = StaticHeap::new();
        &HEAP
    });

    set_implementations(Implementations {
        _sbrk: Some(sbrk_with_static_heap),
        _write: Some(write_with_debug_put_char),
        ..Default::default()
    })
}

mod rand_env {
    use core::cell::RefCell;

    use rand::rngs::SmallRng;
    use rand::{RngCore, SeedableRng};

    #[cfg(not(target_thread_local))]
    compile_error!("");

    #[thread_local]
    static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);

    pub fn seed_insecure_dummy_rng(seed: u64) {
        assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
    }

    pub fn insecure_dummy_rng(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        // HACK
        if RNG.borrow().is_none() {
            seed_insecure_dummy_rng(0);
        }
        RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
        Ok(())
    }

    getrandom::register_custom_getrandom!(insecure_dummy_rng);

    // https://github.com/rust-lang/compiler-builtins/pull/563
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    #[no_mangle]
    pub extern "C" fn __bswapsi2(u: u32) -> u32 {
        ((u & 0xff000000) >> 24)
            | ((u & 0x00ff0000) >> 8)
            | ((u & 0x0000ff00) << 8)
            | ((u & 0x000000ff) << 24)
    }
}
