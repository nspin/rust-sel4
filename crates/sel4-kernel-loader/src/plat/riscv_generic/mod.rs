//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use crate::plat::Plat;

#[unsafe(no_mangle)]
static mut hsm_exists: i32 = 0;

#[unsafe(no_mangle)]
static mut next_logical_core_id: i32 = 1;

#[unsafe(no_mangle)]
static mut start_core_by_logical_id: i32 = 0;

#[unsafe(no_mangle)]
static mut secondary_core_sp: usize = 0;

unsafe extern "C" {
    pub(crate) fn secondary_harts();
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {
        assert!(get_hsm_exists());
    }

    fn put_char(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn put_char_without_synchronization(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn start_secondary_core(core_id: usize, sp: usize) {
        unsafe {
            let _ = sbi::hart_state_management::hart_start(
                core_id,
                sbi::PhysicalAddress::new(secondary_harts as *const () as usize),
                core_id,
            );
            AtomicUsize::from_ptr(ptr::addr_of_mut!(secondary_core_sp)).store(sp, Ordering::SeqCst);
            AtomicI32::from_ptr(ptr::addr_of_mut!(start_core_by_logical_id))
                .store(core_id.try_into().unwrap(), Ordering::SeqCst);
        }
    }
}

fn get_hsm_exists() -> bool {
    unsafe { hsm_exists != 0 }
}
