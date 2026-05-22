//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(dead_code)]

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
    for b in buf {
        putc(b);
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
