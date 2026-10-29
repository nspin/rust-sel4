//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config_macros::sel4_cfg_select;

sel4_cfg_select! {
    ARCH_AARCH64 => {
        #[path = "aarch64/mod.rs"]
        mod imp;
    }
    ARCH_AARCH32 => {
        #[path = "aarch32/mod.rs"]
        mod imp;
    }
    _ => {}
}

// HACK for rustfmt
#[cfg(false)]
mod aarch32;
#[cfg(false)]
mod aarch64;

pub(crate) use imp::*;
