//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_root_task::{root_task, panicking};

#[root_task(stack_size = 64 * 1024, heap_size = 64 * 1024)]
// #[root_task(stack_size = 128 * 1024, heap_size = 64 * 1024)]
// #[root_task(heap_size = 64 * 1024)]
fn main(_: &sel4::BootInfoPtr) -> ! {

    sel4_root_task::print_sp("main");
    let r = panicking::catch_unwind(|| {
        sel4_root_task::print_sp("inside catch unwind");
        panic!("uh oh");
    });
    assert!(r.is_err());
    sel4_root_task::print_sp("after catch");
    // panic!("uh oh");

    sel4::debug_println!("TEST_PASS");
    sel4::init_thread::suspend_self()
}
