//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub(crate) fn putc(c: u8) {
    semihosting::sys::arm_compat::sys_writec(c);
}

pub(crate) fn puts(s: &str) {
    for c in s.bytes() {
        putc(c);
    }
}

fn u64_to_hex<'a>(mut value: usize, buf: &'a mut [u8; 16]) -> &'a str {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    for i in (0..16).rev() {
        buf[i] = HEX[(value & 0xf) as usize];
        value >>= 4;
    }

    // SAFETY:
    // only ASCII hex chars were written
    unsafe { core::str::from_utf8_unchecked(buf) }
}

pub(crate) fn putx(l: &str, v: usize) {
    let mut buf = [0; 16];
    let s = u64_to_hex(v, &mut buf);
    puts(l);
    puts(": ");
    puts(s);
    puts("\n");
}
