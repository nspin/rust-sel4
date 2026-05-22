//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

pub(crate) fn putc(c: u8) {
    semihosting::sys::arm_compat::sys_writec(c);
}

pub(crate) fn puts(s: &str) {
    for c in s.bytes() {
        putc(c);
    }
}

struct DebugWrite;

impl fmt::Write for DebugWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            putc(c);
        }
        Ok(())
    }
}

pub(crate) fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut DebugWrite, args).unwrap_or_else(|err| panic!("write error: {:?}", err))
}

macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::io::debug_print_helper(format_args!($($arg)*)));
}

macro_rules! debug_println {
    () => ($crate::io::debug_println!(""));
    ($($arg:tt)*) => ($crate::io::debug_print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use debug_print;
pub(crate) use debug_println;
