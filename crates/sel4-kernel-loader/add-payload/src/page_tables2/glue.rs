//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use super::regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
use super::scheme::{RawDescriptor, Scheme};
use super::table::{LeafLocation, MkLeafFn, RegionContent, Table};

pub type Region = AbstractRegion<Option<RegionContent>>;
pub type Regions = AbstractRegions<Option<RegionContent>>;
pub type RegionsBuilder = AbstractRegionsBuilder<Option<RegionContent>>;

impl RegionsBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::new_with_background(Region::invalid(SchemeHelpers::<T>::virt_bounds()))
    }
}

impl Regions {
    pub fn construct_table(&self) -> Table {
        Table::construct(self)
    }
}

impl Region {
    pub fn valid(range: Range<u64>, mk_leaf: impl MkLeafFn + 'static) -> Self {
        Self {
            range,
            content: Some(RegionContent::new(mk_leaf)),
        }
    }

    pub fn invalid(range: Range<u64>) -> Self {
        Self {
            range,
            content: None,
        }
    }
}

impl LeafLocation {
    pub fn map(&self, vaddr_to_paddr: impl FnOnce(u64) -> u64) -> RawDescriptor {
        SchemeHelpers::<T>::leaf_descriptor_from_paddr_with_check(
            (vaddr_to_paddr)(self.vaddr()),
            self.level(),
        )
    }

    pub fn map_identity(&self) -> RawDescriptor {
        self.map(|vaddr| vaddr)
    }
}
