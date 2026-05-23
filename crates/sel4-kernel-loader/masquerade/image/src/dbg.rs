//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(dead_code)]

use core::fmt;

pub(crate) fn putc(c: u8) {
    semihosting::sys::arm_compat::sys_writec(c);
}

pub(crate) fn puts(s: &str) {
    for c in s.bytes() {
        putc(c);
    }
}

pub(crate) fn putx(v: usize) {
    let mut buf = [0; 16];
    word_to_hex(v, &mut buf);
    for c in buf {
        putc(c);
    }
}

fn word_to_hex(mut v: usize, buf: &mut [u8; 16]) {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";

    for slot in buf.iter_mut().rev() {
        *slot = DIGITS[v & 0xf];
        v >>= 4;
    }
}

pub(crate) fn putv(n: &str, v: usize) {
    puts(n);
    puts(": ");
    putx(v);
    puts("\n");
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

macro_rules! print {
    ($($arg:tt)*) => ($crate::dbg::debug_print_helper(format_args!($($arg)*)));
}

macro_rules! println {
    () => ($crate::dbg::println!(""));
    ($($arg:tt)*) => ($crate::dbg::print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use print;
pub(crate) use println;
