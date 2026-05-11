//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use std::ops::Range;

use crate::page_tables::{LeafLocation, Region, RegionsBuilder, Scheme, SchemeHelpers, schemes};
use crate::platform_info::PlatformInfoForBuildSystem;

// TODO must be T::align_of_level(0)
pub const ALIGN: u64 = 4096;

pub fn mk_loader_map<S: SchemeExt + 'static>(
    vaddr: u64,
    platform_info: &PlatformInfoForBuildSystem,
) -> (Vec<u8>, u64) {
    let device_range_end = S::device_range_end();

    let mut regions = RegionsBuilder::<S>::new();
    regions = regions.insert(Region::valid(
        0..device_range_end,
        S::mk_device_leaf_for_loader_map,
    ));
    eprintln!("QQA {:#x?}", device_range_end);
    for range in platform_info.memory.iter() {
        eprintln!("QQR {:#x?}", range);
        regions = regions.insert(Region::valid(
            range.clone(),
            S::mk_normal_leaf_for_loader_map,
        ));
    }


    let (entries, root_vaddr) = regions.build().construct_table().embed(vaddr);
    let bytes = {
        let mut v = vec![];
        for entry in entries.iter() {
            v.extend(S::entry_to_bytes(entry));
        }
        v
    };
    (bytes, root_vaddr)
}

pub fn mk_kernel_map<S: SchemeExt + 'static>(
    vaddr: u64,
    kernel_virt_addr_range: Range<u64>,
    kernel_phys_to_virt_offset: u64,
) -> (Vec<u8>, u64) {
    let virt_start = kernel_virt_addr_range.start;
    let virt_end = kernel_virt_addr_range.end;
    let virt_map_end = virt_end.next_multiple_of(1 << SchemeHelpers::<S>::largest_leaf_size_bits());

    eprintln!("XXX {:#x?}", virt_start);
    eprintln!("XXX {:#x?}", virt_map_end);

    let regions = RegionsBuilder::<S>::new()
        .insert(Region::valid(
            0..virt_start,
            S::mk_identity_leaf_for_kernel_map,
        ))
        .insert(Region::valid(virt_start..virt_map_end, move |loc| {
            S::mk_kernel_leaf_for_kernel_map(kernel_phys_to_virt_offset, loc)
        }));

    let (entries, root_vaddr) = regions.build().construct_table().embed(vaddr);
    let bytes = {
        let mut v = vec![];
        for entry in entries.iter() {
            v.extend(S::entry_to_bytes(entry));
        }
        v
    };
    (bytes, root_vaddr)
}

pub trait SchemeExt: Scheme {
    fn mk_normal_leaf_for_loader_map(_loc: LeafLocation) -> Self::LeafDescriptor {
        unimplemented!()
    }

    fn mk_device_leaf_for_loader_map(_loc: LeafLocation) -> Self::LeafDescriptor {
        unimplemented!()
    }

    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor;

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor;

    fn device_range_end() -> u64 {
        unimplemented!()
    }
}

impl SchemeExt for schemes::AArch64 {
    fn mk_normal_leaf_for_loader_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch64>()
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(AARCH64_NORMAL_SHAREABILITY)
    }

    fn mk_device_leaf_for_loader_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch64>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    }

    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch64>()
            .set_access_flag(true)
            .set_attribute_index(0) // select MT_DEVICE_nGnRnE
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<schemes::AArch64>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
            .set_access_flag(true)
            .set_attribute_index(4) // select MT_NORMAL
            .set_shareability(AARCH64_NORMAL_SHAREABILITY)
    }

    fn device_range_end() -> u64 {
        1 << 39
    }
}

// TODO
const AARCH64_NORMAL_SHAREABILITY: u64 = 0;
// if todo!("MAX_NUM_NODES") {
//     0b11
// } else {
//     0b00
// };

impl SchemeExt for schemes::AArch32 {
    fn mk_normal_leaf_for_loader_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch32>()
            .set_access_flag(true)
            .set_attributes(0b101, false, true)
            .set_shareability(true)
    }

    fn mk_device_leaf_for_loader_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch32>().set_access_flag(true)
    }

    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<schemes::AArch32>().set_access_flag(true)
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<schemes::AArch32>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
            .set_access_flag(true)
            .set_shareability(true)
    }

    fn device_range_end() -> u64 {
        SchemeHelpers::<Self>::virt_bounds().end
    }
}

impl SchemeExt for schemes::RiscV64Sv39 {
    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<Self>()
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<Self>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
    }
}

impl SchemeExt for schemes::RiscV32Sv32 {
    fn mk_identity_leaf_for_kernel_map(loc: LeafLocation) -> Self::LeafDescriptor {
        loc.map_identity::<Self>()
    }

    fn mk_kernel_leaf_for_kernel_map(
        phys_to_virt_offset: u64,
        loc: LeafLocation,
    ) -> Self::LeafDescriptor {
        loc.map::<Self>(|vaddr| virt_to_phys(vaddr, phys_to_virt_offset))
    }
}

fn virt_to_phys(vaddr: u64, phys_to_virt_offset: u64) -> u64 {
    vaddr.wrapping_sub(phys_to_virt_offset)
}
