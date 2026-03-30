//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{NullHandler, protection_domain};

sel4_test_sentinels::embed_sdf_script!("../../system.py");

#[protection_domain]
fn init() -> NullHandler {
    sel4_test_sentinels::indicate_success();
    NullHandler::new()
}
