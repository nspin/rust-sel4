//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;

pub(crate) const fn int_to_usize(x: u32) -> usize {
    const _: () = assert!(size_of::<u32>() <= size_of::<usize>());
    x as usize
}

pub(crate) fn status_to_bool(status: c_int) -> bool {
    match status {
        0 => true,
        -1 => false,
        _ => panic!("unexpected status: {status}"),
    }
}

/// Supertrait of role traits, so that only this crate can define roles.
pub trait Sealed {}
