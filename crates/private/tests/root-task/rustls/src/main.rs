//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(cfg_target_thread_local)]
#![feature(never_type)]
#![feature(thread_local)]

extern crate alloc;

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_newlib as _;
use sel4_root_task::{debug_println, root_task};

mod tests;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .write(|s| debug_println!("{}", s))
    .build();

const HEAP_SIZE: usize = 1024 * 1024;

#[root_task(heap_size = HEAP_SIZE)]
fn main(_: &sel4::BootInfo) -> ! {
    LOGGER.set().unwrap();
    run_tests();
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}

fn run_tests() {
    tests::run().unwrap();
}

mod rand_env {
    use core::cell::RefCell;

    use rand::rngs::SmallRng;
    use rand::{RngCore, SeedableRng};

    #[cfg(not(target_thread_local))]
    compile_error!("");

    #[thread_local]
    static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);

    pub fn seed_insecure_dummy_rng(seed: u64) {
        assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
    }

    pub fn insecure_dummy_rng(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        // HACK
        if RNG.borrow().is_none() {
            seed_insecure_dummy_rng(0);
        }
        RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
        Ok(())
    }

    getrandom::register_custom_getrandom!(insecure_dummy_rng);

    // https://github.com/rust-lang/compiler-builtins/pull/563
    #[no_mangle]
    pub extern "C" fn __bswapsi2(u: u32) -> u32 {
        ((u & 0xff000000) >> 24)
            | ((u & 0x00ff0000) >> 8)
            | ((u & 0x0000ff00) << 8)
            | ((u & 0x000000ff) << 24)
    }
}
