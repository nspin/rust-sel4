//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::arch::{naked_asm};
use core::mem;
use core::ptr;
use core::slice;

extern crate sel4_no_panic;

// RUSTFLAGS="-Ccode-model=tiny -Cforce-frame-pointers=no -Cforce-unwind-tables=no -Clink-arg=--no-eh-frame-hdr"

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
    fn to_usize(&self) -> usize {
        usize::from_be_bytes(self.bytes)
    }

    fn to_mut_ptr(&self) -> *mut u8 {
        self.to_usize() as *mut u8
    }
}

type PayloadEntryFn = extern "C" fn(usize) -> !;

struct Payload {
    entry: usize,
    regions: &'static [Region],
    data: *const u8,
}

impl Payload {
    unsafe fn deserialize(start: *const u8) -> Self {
        let mut de = Deserializer::new(start.cast::<BigEndianWord>());
        let entry = unsafe { de.next() }.to_usize();
        let num_regions = unsafe { de.next() }.to_usize();
        let (regions, data) = unsafe { de.rest(num_regions) };
        Self { entry, regions, data }
    }

    unsafe fn deploy(&self) -> Result<usize, Abort> {
        for region in self.regions {
            unsafe {
                let src = self.data.add(region.offset.to_usize());
                let filesz = region.filesz.to_usize();
                ptr::copy(src, region.vaddr.to_mut_ptr(), region.filesz.to_usize());
                ptr::write_bytes(region.vaddr.to_mut_ptr().add(filesz), 0, region.memsz.to_usize() - filesz);
            }
        }
        Ok(self.entry)
    }
}

#[repr(C)]
struct Region {
    vaddr: BigEndianWord,
    offset: BigEndianWord,
    filesz: BigEndianWord,
    memsz: BigEndianWord,
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

unsafe extern "C" {
    safe static _payload_start: usize;
}

fn get_payload() -> Payload {
    unsafe {
        Payload::deserialize(ptr::addr_of!(_payload_start).cast::<u8>())
    }
}

fn main(dtb_addr: usize) -> Result<Never, Abort> {
    let entry = unsafe {
        get_payload().deploy()?
    };
    let entry = unsafe {
        mem::transmute::<usize, PayloadEntryFn>(entry)
    };
    (entry)(dtb_addr)
}
