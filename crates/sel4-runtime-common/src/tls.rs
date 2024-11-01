//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;

use sel4_elf_header::PT_TLS;
use sel4_panicking_env::abort;

#[allow(unused_imports)]
use sel4_initialize_tls::{SetThreadPointerFn, UncheckedTlsImage, DEFAULT_SET_THREAD_POINTER_FN};

pub use sel4_initialize_tls::{ContArg, ContFn, Region};

use crate::locate_phdrs;

extern "C" {
    static mut __tls_start: u8;
    static mut __tls_end: u8;
}

#[allow(unused_unsafe)]
fn tls_region() -> Region {
    let start = unsafe { ptr::addr_of_mut!(__tls_start) };
    let end = unsafe { ptr::addr_of_mut!(__tls_end) };
    Region::new(start, end as usize - start as usize)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn initialize_tls_on_stack_and_continue(cont_fn: ContFn, cont_arg: *mut ContArg) -> ! {
    let phdr = locate_phdrs()
        .iter()
        .find(|phdr| phdr.p_type == PT_TLS)
        .unwrap_or_else(|| abort!("no PT_TLS segment"));
    let unchecked = UncheckedTlsImage {
        vaddr: phdr.p_vaddr,
        filesz: phdr.p_filesz,
        memsz: phdr.p_memsz,
        align: phdr.p_align,
    };
    let checked = unchecked
        .check()
        .unwrap_or_else(|_| abort!("invalid TLS image: {unchecked:#x?}"));
    let thread_pointer = checked
        .initialize_exact_reservation_region(&tls_region())
        .unwrap_or_else(|_| abort!("invalid TLS reservation"));
    CHOSEN_SET_THREAD_POINTER_FN(thread_pointer);
    cont_fn(cont_arg)
}

sel4::sel4_cfg_if! {
    if #[sel4_cfg(all(ARCH_X86_64, SET_TLS_BASE_SELF))] {
        const CHOSEN_SET_THREAD_POINTER_FN: SetThreadPointerFn = set_thread_pointer_via_syscall;

        unsafe extern "C" fn set_thread_pointer_via_syscall(val: usize) {
            sel4::set_tls_base(val);
        }
    } else {
        const CHOSEN_SET_THREAD_POINTER_FN: SetThreadPointerFn = DEFAULT_SET_THREAD_POINTER_FN;
    }
}

#[cfg(target_arch = "arm")]
#[no_mangle]
extern "C" fn __aeabi_read_tp() -> usize {
    let mut val: usize;
    unsafe {
        core::arch::asm!("mrc p15, 0, {val}, c13, c0, 2", val = out(reg) val); // tpidrurw
    }
    val
}
