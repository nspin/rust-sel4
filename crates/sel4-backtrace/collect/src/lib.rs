//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entry {
    pub stack_frame: StackFrame,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StackFrame {
    pub ip: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Error {
    pub unwind_reason_code: i32,
}

cfg_select! {
    all(feature = "unwinding", panic = "unwind") => {
        use core::ffi::c_void;

        use unwinding::abi::*;

        pub fn collect<F: FnMut(Entry) -> Result<(), E>, E>(f: F) -> Option<Error> {
            struct CallbackData<F1> {
                f: F1,
            }

            extern "C" fn callback<F1: FnMut(Entry) -> Result<(), E1>, E1>(
                unwind_ctx: &UnwindContext,
                arg: *mut c_void,
            ) -> UnwindReasonCode {
                let data = unsafe { &mut *(arg as *mut CallbackData<F1>) };
                let ip = _Unwind_GetIP(unwind_ctx);
                match (data.f)(Entry {
                    stack_frame: StackFrame { ip },
                }) {
                    Ok(()) => UnwindReasonCode::NO_REASON,
                    Err(_) => UnwindReasonCode::FATAL_PHASE2_ERROR,
                }
            }

            let mut data = CallbackData { f };
            let code = _Unwind_Backtrace(callback::<F, E>, &mut data as *mut _ as _);
            match code {
                UnwindReasonCode::END_OF_STACK => None,
                _ => Some(Error {
                    unwind_reason_code: code.0,
                }),
            }
        }
    }
    _ => {
        pub fn collect<F: FnMut(Entry) -> Result<(), E>, E>(_f: F) -> Option<Error> {
            None
        }
    }
}
