//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::sys;

/// Corresponds to `seL4_VCPUReg`.
#[repr(u32)]
#[allow(non_camel_case_types)]
#[sel4_config::cfg_enum]
pub enum VCpuReg {
    // TODO
}

impl VCpuReg {
    pub const fn into_sys(self) -> sys::seL4_VCPUReg::Type {
        self as sys::seL4_VCPUReg::Type
    }
}
