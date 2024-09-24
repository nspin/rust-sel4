//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_root_task::{root_task, panicking};

#[root_task]
// #[root_task(heap_size = 64 * 1024)]
fn main(_: &sel4::BootInfoPtr) -> ! {

    let r = panicking::catch_unwind(|| {
        panic!("uh oh");
    });
    assert!(r.is_err());
    // panic!("uh oh");

    sel4::debug_println!("TEST_PASS");
    sel4::init_thread::suspend_self()
}
