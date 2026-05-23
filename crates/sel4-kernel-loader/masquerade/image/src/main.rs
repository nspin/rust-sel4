//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// RUSTFLAGS="-Crelocation-model=pie -Ccode-model=tiny"
// -Z build-std=core,compiler_builtins (to ensure all is pie)
// --release

#![no_std]
#![no_main]

use core::arch::asm;
use core::arch::naked_asm;
use core::fmt::Write;
use core::mem;
use core::panic::PanicInfo;
use core::ptr;

// extern crate sel4_no_panic;

mod caches;
mod dbg;
mod payload;

use dbg::D;
use fdt::Fdt;
use payload::Payload;

#[unsafe(link_section = ".text.header")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn _start() -> ! {
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
                
                mov     x19, x0                 // save dtb addr

                adrp    x9, _start              // set sp = _start + image size
                add     x9, x9, :lo12:_start
                adrp    x10, .Limage_size
                add     x10, x10, :lo12:.Limage_size
                ldr     x10, [x10]
                add     x9, x9, x10
                mov     sp, x9

                mov     x0, #0
                bl      {apply_relocations}

                mov     x0, x19                 // dtb addr
                bl      {main}

            .Lmain_fallthrough:
                wfe
                b       .Lmain_fallthrough
        "#,
        apply_relocations = sym apply_relocations,
        main = sym main,
    }
}

#[unsafe(naked)]
extern "C" fn apply_relocations(link_base: usize) -> ! {
    naked_asm! {
        r#"
                adrp    x1, _start
                add     x1, x1, :lo12:_start

                sub     x10, x1, x0          // delta = runtime - link

                adrp    x2, __rela_start
                add     x2, x2, :lo12:__rela_start

                adrp    x3, __rela_end
                add     x3, x3, :lo12:__rela_end

            .Lloop:
                cmp     x2, x3
                b.hs    .Ldone

                ldr     x4, [x2, #0]         // r_offset
                ldr     x5, [x2, #8]         // r_info
                ldr     x6, [x2, #16]        // r_addend

                and     x7, x5, #0xffffffff
                mov     x8, #1027            // R_AARCH64_RELATIVE
                cmp     x7, x8
                b.ne    .Lbad_reloc

                add     x9, x4, x10          // patch_addr = r_offset + delta
                add     x6, x6, x10          // value      = r_addend + delta
                str     x6, [x9]

                add     x2, x2, #24
                b       .Lloop

            .Ldone:
                dsb     sy
                isb
                ret

            .Lbad_reloc:
                wfe
                b       .Lbad_reloc
        "#,
    }
}

extern "C" fn main(dtb_addr: usize) {
    inspect_dtb(dtb_addr);
    let payload = get_payload();
    let entry_addr = unsafe { payload.deploy() };
    let entry_fn = unsafe { mem::transmute::<usize, EntryFn>(entry_addr) };
    (entry_fn)(dtb_addr)
}

type EntryFn = extern "C" fn(usize) -> !;

fn get_payload() -> Payload {
    unsafe { Payload::deserialize(ptr::addr_of!(_payload_start)) }
}

unsafe extern "C" {
    static _payload_start: u8;
}

fn inspect_dtb(dtb_addr: usize) {
    let fdt = unsafe { Fdt::from_ptr(dtb_addr as *const u8) }.unwrap_or_else(|err| panic!("{err}"));
    for r in fdt.memory().regions() {
        writeln!(D, "region: {:#x}", r.starting_address.addr()).unwrap();
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let _ = writeln!(D, "{info}");
    loop {
        unsafe { asm!("wfe") }
    }
}
