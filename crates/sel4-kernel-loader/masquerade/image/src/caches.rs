//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// #[inline(always)]
fn cache_line_size() -> usize {
    let ctr: u64;
    // crate::io::putx("xxx", 0);
    unsafe {
        core::arch::asm!("mrs {0}, ctr_el0", out(reg) ctr, options(nomem, nostack));
    }

    // DminLine is bits [19:16], log2(words per D-cache line).
    let s = 4usize << ((ctr >> 16) & 0xf);
    // s
    // crate::dbg::putv("s", s);
    64
}

pub(crate) unsafe fn clean_dcache_range(start: usize, len: usize) {
    let line = cache_line_size();
    let end = start.saturating_add(len);
    let mut p = start & !(line - 1);

    while p < end {
        unsafe {
            core::arch::asm!("dc cvac, {0}", in(reg) p, options(nostack));
        }
        p += line;
    }

    unsafe {
        core::arch::asm!("dsb sy", options(nostack));
    }
}

pub(crate) unsafe fn invalidate_icache_all() {
    unsafe {
        core::arch::asm!("ic iallu", "dsb sy", "isb", options(nostack));
    }
}
