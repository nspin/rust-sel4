//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Handler, pd_name, protection_domain, match_handler};
use sel4_test_microkit::{embed_sdf_xml, upcast_handler};

embed_sdf_xml!("../../../x.system");

mod client;
mod server;

// #[protection_domain(heap_size = 0x10_000)]
// fn init() -> impl Handler {
//     match pd_name().unwrap() {
//         "client" => upcast_handler(client::init()),
//         "server" => upcast_handler(server::init()),
//         _ => unreachable!(),
//     }
// }

match_handler! {
    #[protection_domain(heap_size = 0x10_000)]
    fn init {
        "client" => client::init(),
        "server" => server::init(),
    }
}
