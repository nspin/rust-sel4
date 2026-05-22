//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::slice;

use crate::caches::{clean_dcache_range, invalidate_icache_all};

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
        unsafe {
            let mut cursor = start.cast::<BigEndianWord>();
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
            let vaddr = region.vaddr.to_mut_ptr();
            let filesz = region.filesz.to_usize();
            let memsz = region.memsz.to_usize();
            unsafe {
                let src = self.data.add(region.offset.to_usize());
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

unsafe fn deserialize_word(cursor: &mut *const BigEndianWord) -> usize {
    unsafe {
        let word = cursor.read();
        *cursor = cursor.add(1);
        word.to_usize()
    }
}
