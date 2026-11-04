//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Handler, protection_domain};
use sel4_microkit_driver_adapters::serial::driver::HandlerImpl;

#[cfg(feature = "board-qemu_virt_aarch64")]
use sel4_pl011_driver::Driver;

#[protection_domain]
fn init() -> impl Handler {
    let resources = sddf::device_resources!();
    let mmio = resources.region(0);
    let driver = unsafe { Driver::new(mmio.region().vaddr().as_ptr().cast()) };
    let config = sddf::config!(".serial_driver_config", sddf::sys::serial_driver_config_t);
    unsafe { HandlerImpl::new(driver, &resources, &config) }
}
