//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;
use core::mem::{self, MaybeUninit};

use unwinding::abi::*;

use super::{drop_panic, foreign_exception, RUST_EXCEPTION_CLASS};
use crate::Payload;

struct CurrentException {
    exception_present: bool,
    exception: MaybeUninit<UnwindException>,
}

#[cfg(not(target_thread_local))]
compile_error!("");

#[thread_local]
static CURRENT_PAYLOAD: RefCell<Option<Payload>> = RefCell::new(None);

#[thread_local]
static mut CURRENT_EXCEPTION: CurrentException = CurrentException {
    exception_present: false,
    exception: MaybeUninit::uninit(),
};

pub(crate) fn panic_cleanup(exception: *mut u8) -> Payload {
    sel4_panicking_env::debug_println!("AAA 4");
    let exception = exception as *mut UnwindException;
    unsafe {
        if (*exception).exception_class != RUST_EXCEPTION_CLASS {
            _Unwind_DeleteException(exception);
            foreign_exception()
        } else {
            sel4_panicking_env::debug_println!("AAA 5");
            CURRENT_EXCEPTION.exception_present = false;
            CURRENT_PAYLOAD.replace(None).unwrap()
        }
    }
}

pub(crate) fn start_panic(payload: Payload) -> i32 {
    extern "C" fn exception_cleanup(
        _unwind_code: UnwindReasonCode,
        _exception: *mut UnwindException,
    ) {
        drop_panic()
    }
    sel4_panicking_env::debug_println!("AAA 1");

    let mut exception = unsafe { mem::zeroed::<UnwindException>() };
    exception.exception_class = RUST_EXCEPTION_CLASS;
    exception.exception_cleanup = Some(exception_cleanup);

    assert!(CURRENT_PAYLOAD.replace(Some(payload)).is_none());

    sel4_panicking_env::debug_println!("AAA 2");
    unsafe {
        assert!(!CURRENT_EXCEPTION.exception_present);
        CURRENT_EXCEPTION = CurrentException {
            exception_present: true,
            exception: MaybeUninit::new(exception),
        };
        sel4_panicking_env::debug_println!("AAA 3");
        let x = CURRENT_EXCEPTION.exception.assume_init_mut();
        sel4_panicking_env::debug_println!("AAA 4");
        _Unwind_RaiseException(x).0
    }
}
