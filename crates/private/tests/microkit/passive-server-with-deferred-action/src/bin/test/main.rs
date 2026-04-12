//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use core::convert::Infallible;

use alloc::boxed::Box;

use sel4_microkit::{Handler, pd_name, protection_domain};

sel4_test_microkit::embed_sdf_xml!("../../../x.system");

mod client;
mod server;

#[protection_domain(heap_size = 0x10_000)]
fn init() -> Box<dyn Handler<Error = Infallible> + 'static> {
    match pd_name().unwrap() {
        "client" => Box::new(client::init()),
        "server" => Box::new(server::init()),
        _ => unreachable!(),
    }
}
