//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::arch::{asm, naked_asm};
use core::{num, ptr, slice};

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
        Err(Abort) => (),
    }
}

enum Never {}

struct Abort;

#[repr(C)]
struct BigEndianWord {
    _align: [usize; 0],
    bytes: [u8; size_of::<usize>()],
}

impl BigEndianWord {
    fn get(&self) -> usize {
        usize::from_be_bytes(self.bytes)
    }
}

struct Payload {
    entry: usize,
    regions: &'static [Region],
    data: *const u8,
}

#[repr(C)]
struct Region {
    vaddr: BigEndianWord,
    offset: BigEndianWord,
    filesz: BigEndianWord,
    memsz: BigEndianWord,
}

unsafe extern "C" {
    safe static _payload_start: usize;
}

struct Deserializer {
    cursor: *const BigEndianWord,
}

impl Deserializer {
    fn new(start: *const BigEndianWord) -> Self {
        Self {
            cursor: start,
        }
    }

    unsafe fn next(&mut self) -> BigEndianWord {
        unsafe {
            let word = self.cursor.read();
            self.cursor = self.cursor.add(1);
            word
        }
    }

    unsafe fn rest(self, num_regions: usize) -> (&'static [Region], *const u8) {
        let p = self.cursor.cast::<Region>();
        unsafe {
            let regions = slice::from_raw_parts(p, num_regions);
            let data = p.add(num_regions).cast::<u8>();
            (regions, data)
        }
    }
}

unsafe fn deserialize(start: *const u8) -> Payload {
    let mut de = Deserializer::new(start.cast::<BigEndianWord>());
    let entry = unsafe { de.next() }.get();
    let num_regions = unsafe { de.next() }.get();
    let (regions, data) = unsafe { de.rest(num_regions) };
    Payload { entry, regions, data }

}

fn get_payload() -> Payload {
    unsafe {
        deserialize(ptr::addr_of!(_payload_start).cast::<u8>())
    }
}

fn main(dtb_addr: usize) -> Result<Never, Abort> {
    let payload = get_payload();
    if payload.entry == 0 {
        unsafe { asm!("wfe") };
    }
    Err(Abort)
}
