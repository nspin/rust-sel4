//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use bitfield::{BitMut, BitRangeMut};

pub type RawDescriptor = u64;

pub trait LeafDescriptor {
    fn from_level_paddr(level: u8, paddr: u64) -> Self;
    fn to_raw(self) -> RawDescriptor;
}

#[derive(Debug)]
pub struct AArch64LeafDescriptor {
    raw: RawDescriptor,
}

impl LeafDescriptor for AArch64LeafDescriptor {
    fn from_level_paddr(level: u8, paddr: u64) -> Self {
        let mut raw = paddr;
        raw.set_bit_range(1, 0, if level == 3 { 0b11 } else { 0b01 });
        Self { raw }
    }

    fn to_raw(self) -> RawDescriptor {
        self.raw
    }
}

impl AArch64LeafDescriptor {
    pub fn set_access_flag(mut self, value: bool) -> Self {
        self.raw.set_bit(10, value);
        self
    }

    pub fn set_attribute_index(mut self, index: u64) -> Self {
        assert_eq!(index >> 3, 0);
        self.raw.set_bit_range(4, 2, index);
        self
    }

    pub fn set_shareability(mut self, shareability: u64) -> Self {
        assert_eq!(shareability >> 2, 0);
        self.raw.set_bit_range(9, 8, shareability);
        self
    }
}

#[derive(Debug)]
pub struct AArch32LeafDescriptor {
    level: u8,
    raw: RawDescriptor,
}

impl LeafDescriptor for AArch32LeafDescriptor {
    fn from_level_paddr(level: u8, paddr: u64) -> Self {
        let mut raw = paddr;
        raw.set_bit_range(1, 0, if level == 3 { 0b11 } else { 0b01 });
        Self { level, raw }
    }

    fn to_raw(self) -> RawDescriptor {
        self.raw
    }
}

impl AArch32LeafDescriptor {
    pub fn set_access_flag(mut self, value: bool) -> Self {
        let ix = match self.level {
            0 => 10,
            1 => 4,
            _ => unreachable!(),
        };
        self.raw.set_bit(ix, value);
        self
    }

    pub fn set_attributes(mut self, tex: u32, c: bool, b: bool) -> Self {
        assert_eq!(tex >> 3, 0);
        let (tex_hi, tex_lo) = match self.level {
            0 => (14, 12),
            1 => (8, 6),
            _ => unreachable!(),
        };
        self.raw.set_bit_range(tex_hi, tex_lo, tex);
        self.raw.set_bit(3, c);
        self.raw.set_bit(2, b);
        self
    }

    pub fn set_shareability(mut self, value: bool) -> Self {
        let ix = match self.level {
            0 => 16,
            1 => 10,
            _ => unreachable!(),
        };
        self.raw.set_bit(ix, value);
        self
    }
}

#[derive(Debug)]
pub struct RiscV {
    raw: RawDescriptor,
}

impl LeafDescriptor for RiscV {
    fn from_level_paddr(_level: u8, paddr: u64) -> Self {
        let raw = paddr >> 2;
        Self { raw }
            .set_valid(true)
            .set_read(true)
            .set_write(true)
            .set_execute(true)
    }

    fn to_raw(self) -> RawDescriptor {
        self.raw
    }
}

impl RiscV {
    pub fn set_valid(mut self, value: bool) -> Self {
        self.raw.set_bit(0, value);
        self
    }

    pub fn set_read(mut self, value: bool) -> Self {
        self.raw.set_bit(1, value);
        self
    }

    pub fn set_write(mut self, value: bool) -> Self {
        self.raw.set_bit(2, value);
        self
    }

    pub fn set_execute(mut self, value: bool) -> Self {
        self.raw.set_bit(3, value);
        self
    }
}
