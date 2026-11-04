//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Channel, Handler, protection_domain, var};
use sel4_microkit_driver_adapters::rtc::driver::HandlerImpl;
use sel4_pl031_driver::Driver;

#[protection_domain]
fn init() -> impl Handler {
    let resources = sddf::device_resources!();
    let mmio = resources.region(0);
    let driver = unsafe { Driver::new(mmio.region().vaddr().as_ptr().cast()) };
    let client = Channel::new(*var!(client_channel_id: usize = usize::MAX));
    HandlerImpl::new(driver, client)
}
