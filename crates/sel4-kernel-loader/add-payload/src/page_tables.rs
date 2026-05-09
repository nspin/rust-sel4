//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use std::ops::Range;

use crate::platform_info::PlatformInfoForBuildSystem;

pub fn mk_loader_map(vaddr: u64, platform_info: &PlatformInfoForBuildSystem) -> Vec<u8> {
    todo!()
}

pub fn mk_kernel_map(
    vaddr: u64,
    kernel_phys_addr_range: Range<u64>,
    kernel_phys_to_virt_offset: u64,
) -> Vec<u8> {
    todo!()
}
