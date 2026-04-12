//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Handler, pd_name, protection_domain};

mod client;
mod server;

#[protection_domain]
fn init() -> impl Handler {
    match pd_name() {
        "client" => Box::new(client::init()),
        "server" => Box::new(server::init()),
    }
}
