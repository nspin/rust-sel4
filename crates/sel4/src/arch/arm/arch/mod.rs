//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

sel4_config_macros::cfg_select! {
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

pub(crate) use imp::*;
