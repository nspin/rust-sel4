//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::slice;

use crate::io::*;
use crate::caches::*;

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

pub(crate) struct Payload {
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

impl Payload {
    pub(crate) unsafe fn deserialize(start: *const u8) -> Self {
        let mut de = Deserializer::new(start.cast::<BigEndianWord>());
        // puts("a\n");
        let entry = unsafe { de.next() }.to_usize();
        // puts("b\n");
        let num_regions = unsafe { de.next() }.to_usize();
        let (regions, data) = unsafe { de.rest(num_regions) };
        Self {
            entry,
            regions,
            data,
        }
    }

    pub(crate) unsafe fn deploy(&self) -> usize {
        // putx("data", self.data.addr());
        for region in self.regions {
            // putx("vaddr", region.vaddr.to_usize());
            // putx("offset", region.offset.to_usize());
            // putx("filesz", region.filesz.to_usize());
            // putx("memsz", region.memsz.to_usize());
            // puts("\n");
            unsafe {
                let src = self.data.add(region.offset.to_usize());
                let filesz = region.filesz.to_usize();
                ptr::copy(src, region.vaddr.to_mut_ptr(), region.filesz.to_usize());
                ptr::write_bytes(
                    region.vaddr.to_mut_ptr().add(filesz),
                    0,
                    region.memsz.to_usize() - filesz,
                );
                sync_written_code(region.vaddr.to_usize(), region.memsz.to_usize());
            }
        }
        self.entry
    }
}

struct Deserializer {
    cursor: *const BigEndianWord,
}

impl Deserializer {
    fn new(start: *const BigEndianWord) -> Self {
        Self { cursor: start }
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
