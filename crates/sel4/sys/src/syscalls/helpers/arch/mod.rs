//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;

use crate::seL4_Word;

sel4_config::cfg_select! {
    ARCH_AARCH64 => {
        #[path = "aarch64.rs"]
        mod imp;
    }
    ARCH_AARCH32 => {
        #[path = "aarch32.rs"]
        mod imp;
    }
    ARCH_RISCV => {
        #[path = "riscv.rs"]
        mod imp;
    }
    ARCH_X86_64 => {
        #[path = "x86_64.rs"]
        mod imp;
    }
    _ => {}
}

pub use imp::*;

fn sys_id_to_word(sys_id: c_int) -> seL4_Word {
    sys_id as seL4_Word
}
