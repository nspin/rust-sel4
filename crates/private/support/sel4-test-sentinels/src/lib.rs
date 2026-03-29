//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(core_intrinsics)]
#![allow(internal_features)]

use sel4_panicking_env::{debug_println, register_abort_trap};

const SUCCESS: char = '\u{0006}';
const FAILURE: char = '\u{0015}';

pub fn indicate_success() {
    debug_println!("INDICATE_SUCCESS\n{SUCCESS}\n");
}

fn indicate_failure() {
    debug_println!("INDICATE_FAILURE\n{FAILURE}\n");
}

register_abort_trap! {
    trap
}

fn trap() -> ! {
    indicate_failure();
    core::intrinsics::abort()
}
