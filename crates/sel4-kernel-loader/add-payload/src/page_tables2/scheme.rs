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

pub type RawAttributes = u64;

pub trait ArchAttributes {
    fn from_level_paddr(level: u8, paddr: u64) -> RawAttributes;
}

#[derive(Debug)]
pub struct Attributes<T> {
    _phantom: PhantomData<T>,
    level: u8,
    raw: RawAttributes,
}

impl<T: ArchAttributes> Attributes<T> {
    pub fn from_level_paddr(level: u8, paddr: u64) -> Self {
        Self {
            _phantom: PhantomData,
            level,
            raw: T::from_level_paddr(level, paddr),
        }
    }
}

#[derive(Debug)]
pub enum AArch64 {}

impl ArchAttributes for AArch64{
    fn from_level_paddr(level: u8, paddr: u64) -> RawAttributes {
        desc.set_bit_range(1, 0, if level == 3 { 0b11 } else { 0b01 });
    }
}
