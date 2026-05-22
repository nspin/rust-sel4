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

// extern crate sel4_no_panic;

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

fn main(dtb_addr: usize) {
    // puts("hello\r\n\r\0");
    // pc(b'x');
    // sh_write0(b"hello\n\0");
    // semihosting::eprintln!("xxx");
    semihosting::sys::arm_compat::sys_writec(b'x');
    // let entry = unsafe { get_payload().deploy() };
    // sh_write0(b"xello\n\0");
    // let entry = unsafe { mem::transmute::<usize, EntryFn>(entry) };
    // (entry)(dtb_addr)
}

unsafe extern "C" {
    safe static _payload_start: usize;
}

fn get_payload() -> Payload {
    unsafe { Payload::deserialize(ptr::addr_of!(_payload_start).cast::<u8>()) }
}

type EntryFn = extern "C" fn(usize) -> !;

// #[inline(never)]
// pub fn sh_putc(c: u8) {
//     let args = [c as usize];

//     unsafe {
//         core::arch::asm!(
//             "hlt #0xf000",
//             inout("x0") 0x03usize => _, // SYS_WRITEC
//             inout("x1") args.as_ptr() as usize => _,
//             options(readonly, preserves_flags),
//         );
//     }
// }

// fn putc(c: u8) {
//     let s = [c, 0];
//     sh_write0(&s);
//     loop {}
// }

#[inline(never)]
fn sh_write0(s: &[u8]) {
    unsafe {
        core::arch::asm!(
            "hlt #0xf000",
            in("x0") 0x04usize, // SYS_WRITE0
            in("x1") s.as_ptr() as usize,
            lateout("x0") _,
            lateout("x1") _,
            options(readonly, preserves_flags),
        );
    }
}

// #[inline(never)]
// pub fn sh_putc(c: u8) {
//     let ch = c;

//     unsafe {
//         core::arch::asm!(
//             "hlt #0xf000",
//             inout("x0") 0x03usize => _,
//             inout("x1") (&raw const ch) as usize => _,
//             // Do not use `nomem` or `readonly`.
//             // QEMU reads guest memory through x1.
//             options(preserves_flags),
//         );
//     }
// }

// fn pc(c: u8
// ) {
//     let mut ch = c;
//     unsafe {
//         core::arch::asm!(
//             "hlt #0xf000",
//             in("w0") 3, // OPERATION NUMBER REGISTER
//             in("x1") &mut ch, // PARAMETER REGISTER
//             options(nostack, preserves_flags, readonly),
//         );
//     }
// }

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    semihosting::eprintln!("{info}");
    todo!()
}
