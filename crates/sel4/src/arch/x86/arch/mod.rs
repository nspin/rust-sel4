//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

sel4_config_macros::cfg_select! {
    ARCH_X86_64 => {
        #[path = "x64/mod.rs"]
        mod imp;
    }
    _ => {}
}

pub(crate) use imp::*;
