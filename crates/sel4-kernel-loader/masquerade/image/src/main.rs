//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::arch::naked_asm;
use core::mem;
use core::ptr;

extern crate sel4_no_panic;

mod payload;

use payload::Payload;

// RUSTFLAGS="-Ccode-model=tiny -Cforce-frame-pointers=no -Cforce-unwind-tables=no"

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
    main(dtb_addr)
}

fn main(dtb_addr: usize) -> ! {
    let entry = unsafe { get_payload().deploy() };
    let entry = unsafe { mem::transmute::<usize, EntryFn>(entry) };
    (entry)(dtb_addr)
}

unsafe extern "C" {
    safe static _payload_start: usize;
}

fn get_payload() -> Payload {
    unsafe { Payload::deserialize(ptr::addr_of!(_payload_start).cast::<u8>()) }
}

type EntryFn = extern "C" fn(usize) -> !;
