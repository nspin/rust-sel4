//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Handler, protection_domain};
use sel4_microkit_driver_adapters::timer::driver::HandlerImpl;
use sel4_sp804_driver::Driver;

const FREQ: u64 = 1_000_000;

#[protection_domain]
fn init() -> impl Handler {
    let resources = sddf::device_resources!();
    let mmio = resources.region(0);
    let driver = unsafe { Driver::new(mmio.region().vaddr().as_ptr().cast(), FREQ) };
    HandlerImpl::new(driver, &resources)
}
