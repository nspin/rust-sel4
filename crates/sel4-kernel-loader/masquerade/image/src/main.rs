//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// RUSTFLAGS="-Crelocation-model=pie -Ccode-model=tiny -Cforce-frame-pointers=no -Cforce-unwind-tables=no"

#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::naked_asm;
use core::mem;

extern crate sel4_no_panic;

mod caches;
// mod dbg;
mod payload;

use payload::Payload;

#[unsafe(link_section = ".text.header")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() -> ! {
    naked_asm! {
        r#"
                b       .Lreal_start
                .long   0                       // code1 padding
                .quad   0x00080000              // text_offset: conventional 512 KiB
            .Limage_size:
                .quad   0                       // image_size (to be patched by tool)
                .quad   0                       // flags: LE, page size unspecified, placement old-style
                .quad   0                       // res2
                .quad   0                       // res3
                .quad   0                       // res4
                .long   0x644d5241              // magic: "ARM\x64"
                .long   0                       // res5

                .balign 8

            .Lreal_start:

                adrp    x9, _start              // set sp = _start + image size
                add     x9, x9, :lo12:_start
                adrp    x10, .Limage_size
                add     x10, x10, :lo12:.Limage_size
                ldr     x10, [x10]
                add     x9, x9, x10
                mov     sp, x9

                bl      {main}

            .Lhang:
                wfe
                b       .Lhang
        "#,
        main = sym main,
    }
}

extern "C" fn main(dtb_addr: usize) {
    let payload = get_payload();
    let entry_addr = unsafe { payload.deploy() };
    let entry_fn = unsafe { mem::transmute::<usize, EntryFn>(entry_addr) };
    (entry_fn)(dtb_addr)
}

type EntryFn = extern "C" fn(usize) -> !;

fn get_payload() -> Payload {
    unsafe { Payload::deserialize(get_payload_ptr()) }
}

unsafe extern "C" {
    static _payload_start: u8;
}

// HACK
// ptr::addr_of!(_payload_start)) doesn't work with -Crelocation-model=pie
fn get_payload_ptr() -> *const u8 {
    let p: *const u8;
    unsafe {
        asm!(
            "adrp {tmp}, {s}",
            "add  {tmp}, {tmp}, :lo12:{s}",
            tmp = lateout(reg) p,
            s = sym _payload_start,
            options(nostack, pure, readonly),
        );
    }
    p
}
