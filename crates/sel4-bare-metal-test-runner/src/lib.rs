//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]

use one_shot_mutex::sync::RawOneShotMutex;
use syscalls::{Sysno, syscall};

use sel4_dlmalloc::{StaticDlmalloc, StaticHeap};
use sel4_panicking::catch_unwind;
use sel4_panicking_env::{abort, debug_println};

// Stack

const STACK_SIZE: usize = DEFAULT_STACK_SIZE;

pub const DEFAULT_STACK_SIZE: usize = 1024
    * if cfg!(panic = "unwind") && cfg!(debug_assertions) {
        128
    } else {
        64
    };

sel4_runtime_common::declare_stack!(STACK_SIZE);

// Heap

const HEAP_SIZE: usize = DEFAULT_HEAP_SIZE;

const DEFAULT_HEAP_SIZE: usize = 0x10_000;

static STATIC_HEAP: StaticHeap<{ HEAP_SIZE }> = StaticHeap::new();

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmalloc<RawOneShotMutex> =
    StaticDlmalloc::new(STATIC_HEAP.bounds());

// Entrypoint

sel4_runtime_common::declare_entrypoint! {
    () -> ! {
        entry()
    }
}

// for stack provided by OS
// #[unsafe(no_mangle)]
// extern "C" fn _start() {
//     entry()
// }

fn entry() -> ! {
    let result = catch_unwind(move || main());
    match result {
        Ok(_) => exit(0),
        Err(_) => abort!("uncaught panic in main"),
    }
}

// Printing

sel4_panicking_env::register_debug_put_char! {
    debug_put_char
}

fn debug_put_char(c: u8) {
    let _ = unsafe { syscall!(Sysno::write, 1, &raw const c, 1) };
}

// Exiting

fn exit(status: u8) -> ! {
    let r = unsafe { syscall!(Sysno::exit, status) };
    if let Err(err) = r {
        abort!("exit syscall returned error: {}", err)
    }
    unreachable!()
}

sel4_panicking_env::register_abort_trap! {
    exit_abort
}

fn exit_abort() -> ! {
    exit(1)
}

// Main

fn main() -> ! {
    debug_println!("Hello, World!");
    todo!()
}
