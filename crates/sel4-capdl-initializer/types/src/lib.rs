//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use core::ops::Range;

use rkyv::Archive;
use rkyv::primitive::{ArchivedU16, ArchivedU32, ArchivedU64};

mod cap_table;
mod embed_fill;
mod frame_init;
mod spec;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{HasArchivedCapTable, HasCapTable};
pub use frame_init::*;
pub use spec::*;

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // //

pub type InputSpec = Spec<Fill<FillEntryContentFileOffset>>;

pub type SpecForInitializer = Spec<FrameInit>;

// // //

pub trait CramUsize: Copy + TryFrom<usize> + TryInto<usize> {
    fn into_usize(self) -> usize {
        self.try_into().unwrap_or_else(|_| panic!())
    }

    fn from_usize(x: usize) -> Self {
        Self::try_from(x).unwrap_or_else(|_| panic!())
    }

    fn into_usize_range(range: &Range<Self>) -> Range<usize> {
        range.start.into_usize()..range.end.into_usize()
    }

    fn from_usize_range(range: &Range<usize>) -> Range<Self> {
        Self::from_usize(range.start)..Self::from_usize(range.end)
    }
}

impl CramUsize for u8 {}
impl CramUsize for u16 {}
impl CramUsize for u32 {}
impl CramUsize for u64 {}

impl CramUsize for ArchivedU16 {}
impl CramUsize for ArchivedU32 {}
impl CramUsize for ArchivedU64 {}

// // //

pub trait ArchiveSimple: Archive + Copy {
    fn into_archived(self) -> Self::Archived;
    fn from_archived(x: Self::Archived) -> Self;
}

macro_rules! impl_archive_simple_using_from_into {
    ($ty:ty) => {
        impl ArchiveSimple for $ty {
            fn into_archived(self) -> Self::Archived {
                self.into()
            }

            fn from_archived(x: Self::Archived) -> Self {
                Self::from(x)
            }
        }
    };
}

impl_archive_simple_using_from_into!(u16);
impl_archive_simple_using_from_into!(u32);
impl_archive_simple_using_from_into!(u64);
