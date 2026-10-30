//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::Infallible;

use sel4_backtrace_collect::collect;
use sel4_backtrace_embedded_debug_info::get_context;
use sel4_backtrace_symbolize::{Options, Symbolizer};
use sel4_root_task::{debug_println, panicking, root_task};

#[root_task(heap_size = 2 << 20)]
fn main(_: &sel4::BootInfoPtr) -> ! {
    let _ = panicking::catch_unwind(|| {
        f();
    });

    sel4_test_root_task::indicate_success()
}

pub fn f() {
    [()].iter().for_each(g);
}

fn g(_: &()) {
    let mut ips = Vec::new();
    let error = collect(|entry| {
        ips.push(entry.stack_frame.ip);
        Ok::<_, Infallible>(())
    });
    assert!(error.is_none());

    debug_println!("");

    for ip in ips.iter() {
        debug_println!("{ip:#018x}");
    }

    debug_println!("");

    // TODO: find a check that works for all platforms
    // assert!(ips.len().abs_diff(10) <= 3);

    let ctx = get_context().unwrap();
    let mut s = String::new();
    let mut symbolizer = Symbolizer::new(&mut s, &ctx, Options::default());
    for ip in ips {
        symbolizer.symbolize(ip.try_into().unwrap()).unwrap();
    }
    debug_println!("{}", s);
}
