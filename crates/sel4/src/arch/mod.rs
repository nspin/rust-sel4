//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

// TODO
// sel4-config doesn't yet play nicely with:
//   - ARCH_ARM
//   - ARCH_RISCV
//   - ARCH_X86

sel4_config::cfg_select! {
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

pub(crate) use imp::*;
