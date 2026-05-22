//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;

#[unsafe(no_mangle)]
fn cache_line_size() -> usize {
    let ctr: u64;
    unsafe {
        asm!("mrs {}, ctr_el0", out(reg) ctr, options(nomem, nostack));
    }

    // DminLine is bits [19:16], log2(words per D-cache line).
    4usize << ((ctr >> 16) & 0xf)
}

pub(crate) unsafe fn clean_dcache_range(start: usize, size: usize) {
    let line = cache_line_size();
    let end = start.saturating_add(size);
    let mut p = start & !(line - 1);

    while p < end {
        unsafe {
            asm!("dc cvac, {}", in(reg) p, options(nostack));
        }
        p += line;
    }

    unsafe {
        asm!("dsb sy", options(nostack));
    }
}

pub(crate) unsafe fn invalidate_icache_all() {
    unsafe {
        asm! {
            r#"
                ic iallu
                dsb sy
                isb
            "#,
            options(nostack),
        };
    }
}
