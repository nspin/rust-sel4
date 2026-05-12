//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;

use bitfield::{BitMut, BitRange, BitRangeMut};

pub type RawEntry = u64;

pub type RawAttributes = u64;

pub trait ArchAttributes {
    fn new(level: u8) -> RawAttributes;
    fn leaf(attrs: &Attributes<Self>, paddr: u64) -> RawEntry;
}

#[derive(Debug)]
pub struct Attributes<T: ?Sized> {
    _phantom: PhantomData<T>,
    level: u8,
    raw: RawAttributes,
}

impl<T: ArchAttributes> Attributes<T> {
    pub fn from_level_paddr(level: u8) -> Self {
        Self {
            _phantom: PhantomData,
            level,
            raw: T::new(level),
        }
    }

    pub fn leaf(&self, paddr: u64) -> RawEntry {
        T::leaf(self, paddr)
    }
}

#[derive(Debug)]
pub enum AArch64 {}

impl ArchAttributes for AArch64{
    fn new(level: u8) -> RawAttributes {
        let mut attrs = 0;
        attrs.set_bit_range(1, 0, if level == 3 { 0b11 } else { 0b01 });
        attrs
    }

    fn leaf(attrs: &Attributes<Self>, paddr: u64) -> RawEntry {
        attrs.raw | paddr
    }
}

impl Attributes<AArch64> {
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
pub enum AArch32 {}

impl ArchAttributes for AArch32{
    fn new(level: u8) -> RawAttributes {
        let mut attrs = 0;
        attrs.set_bit_range(1, 0, 0b10);
        attrs
    }

    fn leaf(attrs: &Attributes<Self>, paddr: u64) -> RawEntry {
        attrs.raw | paddr
    }
}

impl Attributes<AArch64> {
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
