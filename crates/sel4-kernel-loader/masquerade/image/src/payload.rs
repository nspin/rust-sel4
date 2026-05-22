//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::slice;

use crate::caches::{clean_dcache_range, invalidate_icache_all};

pub(crate) struct Payload {
    entry: usize,
    regions: &'static [Region],
    data: *const u8,
}

#[repr(C)]
struct Region {
    vaddr: usize,
    offset: usize,
    filesz: usize,
    memsz: usize,
}

impl Payload {
    pub(crate) unsafe fn deserialize(start: *const u8) -> Self {
        unsafe {
            let mut cursor = start.cast::<usize>();
            let entry = deserialize_word(&mut cursor);
            let num_regions = deserialize_word(&mut cursor);
            let cursor = cursor.cast::<Region>();
            let regions = slice::from_raw_parts(cursor, num_regions);
            let cursor = cursor.add(num_regions);
            let data = cursor.cast::<u8>();
            Self {
                entry,
                regions,
                data,
            }
        }
    }

    pub(crate) unsafe fn deploy(self) -> usize {
        for region in self.regions {
            let vaddr = region.vaddr as *mut u8;
            let filesz = region.filesz;
            let memsz = region.memsz;
            unsafe {
                let src = self.data.add(region.offset);
                ptr::copy(src, vaddr, filesz);
                ptr::write_bytes(vaddr.add(filesz), 0, memsz - filesz);
                clean_dcache_range(vaddr.addr(), memsz);
            }
        }
        unsafe {
            invalidate_icache_all();
        }
        self.entry
    }
}

unsafe fn deserialize_word(cursor: &mut *const usize) -> usize {
    unsafe {
        let word = cursor.read();
        *cursor = cursor.add(1);
        word
    }
}
