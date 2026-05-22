//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::arch::{asm, naked_asm};
use core::{ptr, slice};

extern crate sel4_no_panic;

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.header")]
unsafe extern "C" fn _start() -> ! {
    naked_asm! {
        r#"
                b       .Lreal_start
                .long   0                       // code1 padding
                .quad   0x00080000              // text_offset: conventional 512 KiB
            .Limage_size:
                .quad   0x2d0                   // image_size
                .quad   0                       // flags: LE, page size unspecified, placement old-style
                .quad   0                       // res2
                .quad   0                       // res3
                .quad   0                       // res4
                .ascii  "ARM\x64"               // magic
                .long   0                       // res5 / PE-COFF offset

                .balign 8

            .Lreal_start:

                adrp    x9, _start              // compute sp: _start + image size
                add     x9, x9, :lo12:_start
                adrp    x10, .Limage_size
                add     x10, x10, :lo12:.Limage_size
                ldr     x10, [x10]
                add     x9, x9, 10
                mov     sp, x9

                bl      {rust_entry}

            .Lhang:
                wfe
                b       .Lhang
        "#,
        rust_entry = sym rust_entry,
    }
}

extern "C" fn rust_entry(dtb_addr: usize) {
    match main(dtb_addr) {
        Err(ExplicitPanic) => (),
    }
}

enum Never {}

struct Abort;

unsafe extern "C" {
    safe static _payload_start: usize;
}

fn get_payload() -> &'static [u8] {
    unsafe {
        slice::from_raw_parts(ptr::addr_of!(_payload_start).cast(), 100)
    }
}

fn main(dtb_addr: usize) -> Result<Never, Abort> {
    let payload = get_payload();
    let x = payload.get(10000).ok_or(Abort);
    // if x.as_ref() == Some(0) {
    //     unsafe { asm!("wfe") };
    // }
    Err(Abort)
}
