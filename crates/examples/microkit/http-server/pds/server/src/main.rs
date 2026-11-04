//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::time::Duration;

use one_shot_mutex::sync::OneShotMutex;
use rtcc::DateTimeAccess;
use smoltcp::iface::Config;
use smoltcp::phy::{Device, Medium};
use smoltcp::wire::{EthernetAddress, HardwareAddress};

use sddf::sys::{blk_client_config_t, net_client_config_t, timer_client_config_t};
use sddf_async_block_io::SddfBlockIO;
use sddf_smoltcp::DeviceImpl;
use sel4_async_block_io::{BlockIOAdapter, ByteIOAdapter, ConstantBlockSize};
use sel4_async_block_io_cached::CachedBlockIO;
use sel4_async_block_io_disk::Disk;
use sel4_async_time::Instant;
use sel4_driver_interfaces::timer::Clock;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_microkit::{Channel, Handler, protection_domain, var};
use sel4_microkit_driver_adapters::rtc::client::Client as RtcClient;
use sel4_microkit_driver_adapters::timer::client::Client as TimerClient;
use sel4_newlib as _;

use microkit_http_server_example_server_core::run_server;

mod handler;

use handler::{BlockIO, HandlerImpl};

const BLOCK_CACHE_SIZE_IN_BLOCKS: usize = 32;

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

    let timer_config = sddf::config!(".timer_client_config", timer_client_config_t);
    let net_config = sddf::config!(".net_client_config", net_client_config_t);
    let blk_config = sddf::config!(".blk_client_config", blk_client_config_t);

    let timer_channel = Channel::new(timer_config.driver_id.into());
    let net_rx_channel = Channel::new(net_config.rx.id.into());
    let blk_channel = Channel::new(blk_config.virt.id.into());
    let rtc_channel = Channel::new(*var!(rtc_driver_channel_id: usize = usize::MAX));

    let mut rtc_client = RtcClient::new(rtc_channel);

    let timer_client = Arc::new(OneShotMutex::new(TimerClient::new(
        sddf::timer::Client::from_config(&timer_config),
    )));

    let now_unix_time = Duration::from_secs(
        rtc_client
            .datetime()
            .unwrap()
            .and_utc()
            .timestamp()
            .try_into()
            .unwrap(),
    );

    let now_fn = {
        let timer_client: Arc<_> = timer_client.clone();
        move || Instant::ZERO + timer_client.lock().get_time().unwrap()
    };

    let net_device = unsafe { DeviceImpl::new(&net_config) };

    let net_iface_config = {
        assert_eq!(net_device.capabilities().medium, Medium::Ethernet);
        let mac_address = EthernetAddress(net_config.mac_addr.addr);
        let hardware_addr = HardwareAddress::Ethernet(mac_address);
        let mut this = Config::new(hardware_addr);
        this.random_seed = 0;
        this
    };

    while !unsafe { sddf::blk::is_storage_info_ready(&blk_config) } {
        core::hint::spin_loop();
    }

    let shared_block_io: BlockIO = Rc::new(unsafe { SddfBlockIO::new(&blk_config) });

    HandlerImpl::new(
        timer_channel,
        net_rx_channel,
        blk_channel,
        timer_client,
        net_device,
        net_iface_config,
        shared_block_io.clone(),
        |timers_ctx, network_ctx, spawner| async move {
            let block_io = CachedBlockIO::new(shared_block_io.clone(), BLOCK_CACHE_SIZE_IN_BLOCKS);
            // The sDDF block device's block size comes from its storage information, but MBR
            // partitions are described in 512-byte sectors.
            let disk = Disk::new(BlockIOAdapter::new(
                ByteIOAdapter::new(block_io),
                ConstantBlockSize::<512>,
            ));
            let mbr = disk.read_mbr().await.unwrap().unwrap();
            let partition = disk.partition_mbr(&mbr.partitions[0]);
            let fs_io = ByteIOAdapter::new(Rc::new(partition));
            run_server(
                now_unix_time,
                now_fn,
                timers_ctx,
                network_ctx,
                fs_io,
                spawner,
                CERT_PEM,
                PRIV_PEM,
                MAX_NUM_SIMULTANEOUS_CONNECTIONS,
            )
            .await
        },
    )
}
