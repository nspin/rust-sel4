//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO feature for defaulting to write_with_debug_put_char?

#![no_std]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(sync_unsafe_cell)]

#[allow(unused_imports)]
use core::ffi::{c_char, c_int, c_uint, c_void};

pub mod heap;

mod errno {
    use super::*;

    pub(crate) const ENOENT: c_int = 2;
}

extern "C" {
    #[link_name = "srand"]
    fn newlib_srand(seed: c_uint);
}

pub fn srand(seed: c_uint) {
    unsafe {
        newlib_srand(seed);
    }
}

#[cfg(feature = "_exit")]
mod impl_exit {
    use super::*;

    use sel4_panicking_env::abort;

    #[no_mangle]
    extern "C" fn _exit(rc: c_int) -> ! {
        abort!("_exit({})", rc)
    }
}

#[cfg(feature = "_write")]
mod impl_write {
    use super::*;

    use core::slice;

    use sel4_panicking_env::debug_put_char;

    #[no_mangle]
    extern "C" fn _write(file: c_int, ptr: *const c_char, len: c_int) -> c_int {
        match file {
            1 | 2 => {
                let bytes =
                    unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len.try_into().unwrap()) };
                for b in bytes {
                    debug_put_char(*b);
                }
                len
            }
            _ => {
                #[cfg(feature = "log")]
                {
                    log::warn!("_write({}, {:#x?}, {})", file, ptr, len);
                }
                -errno::ENOENT
            }
        }
    }
}
