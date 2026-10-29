//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config_macros::sel4_cfg_select;

// TODO
// sel4-config doesn't yet play nicely with:
//   - ARCH_ARM
//   - ARCH_RISCV
//   - ARCH_X86

sel4_cfg_select! {
    ARCH_ARM => {
        #[path = "arm/mod.rs"]
        mod imp;
    }
    ARCH_RISCV => {
        #[path = "riscv/mod.rs"]
        mod imp;
    }
    ARCH_X86_64 => {
        #[path = "x86/mod.rs"]
        mod imp;
    }
    _ => {}
}

// HACK for rustfmt
#[cfg(false)]
mod arm;
#[cfg(false)]
mod riscv;
#[cfg(false)]
mod x86;

pub(crate) use imp::*;
