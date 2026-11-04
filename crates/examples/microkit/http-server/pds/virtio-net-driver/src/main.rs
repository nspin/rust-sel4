//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use virtio_drivers::{
    device::net::*,
    transport::{
        DeviceType, Transport,
        mmio::{MmioTransport, VirtIOHeader},
    },
};

use sddf::sys::net_driver_config_t;
use sel4_microkit::{Channel, protection_domain};
use sel4_microkit_driver_adapters::net::driver::HandlerImpl;
use sel4_virtio_hal_impl::HalImpl;
use sel4_virtio_net::DeviceWrapper;

const NET_QUEUE_SIZE: usize = 16;
const NET_BUFFER_LEN: usize = 2048;

const MMIO_REGION: usize = 0;
const DMA_REGION: usize = 1;
const RX_DATA_REGION: usize = 2;
const TX_DATA_REGION: usize = 3;

#[protection_domain(
    heap_size = 512 * 1024,
)]
fn init() -> HandlerImpl<DeviceWrapper<HalImpl, MmioTransport<'static>>> {
    let resources = sddf::device_resources!();
    let config = sddf::config!(".net_driver_config", net_driver_config_t);

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
        assert_eq!(transport.device_type(), DeviceType::Network);
        VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN).unwrap()
    };

    let device_channel = Channel::new(resources.irq(0));

    dev.ack_interrupt();
    device_channel.irq_ack().unwrap();

    unsafe {
        HandlerImpl::new(
            DeviceWrapper::new(dev),
            &config,
            *resources.region(RX_DATA_REGION),
            *resources.region(TX_DATA_REGION),
            &resources,
        )
    }
}
