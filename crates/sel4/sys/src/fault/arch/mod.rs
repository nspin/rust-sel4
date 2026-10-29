//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config_macros::sel4_cfg_select;

sel4_cfg_select! {
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

// HACK for rustfmt
#[cfg(false)]
mod aarch32;
#[cfg(false)]
mod aarch64;
#[cfg(false)]
mod riscv;
#[cfg(false)]
mod x86_64;
