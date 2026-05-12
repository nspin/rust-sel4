//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// mod embed;
// mod glue;
mod regions;
mod scheme;
// mod table;

// pub use glue::{Region, Regions, RegionsBuilder};
pub use regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
pub use scheme::{LeafDescriptor, Level, RawDescriptor, Scheme};
// pub use table::{LeafLocation, MkLeafFn, RegionContent, Table};

pub mod schemes {
    pub use super::scheme::{AArch32LeafDescriptor, AArch64LeafDescriptor, RiscVLeafDescriptor};
}
