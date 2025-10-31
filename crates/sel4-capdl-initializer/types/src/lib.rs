//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::string::String;
use core::ops::Range;

use rkyv::Archive;
use rkyv::primitive::{ArchivedU16, ArchivedU32, ArchivedU64};

mod archived_cap_table;
mod cap_table;
mod frame_init;
mod inspect;
mod inspect_archived;
mod object_name;
mod spec;

mod traverse;

#[cfg(feature = "std")]
mod when_std;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use archived_cap_table::{ArchivedPageTableEntry, HasArchivedCapTable};
pub use cap_table::{HasCapTable, PageTableEntry};
pub use frame_init::{
    ArchivedFillEntry, ArchivedFillEntryContent, ArchivedFillEntryContentBootInfoId,
    ArchivedFrameInit, BytesContent, Content, EmbeddedFrameOffset, Fill, FillEntry,
    FillEntryContent, FillEntryContentBootInfo, FillEntryContentBootInfoId, FrameInit,
    GetEmbeddedFrameOffset, NeverEmbedded,
};
pub use object_name::{ArchivedUnnamed, ObjectName, ObjectNamesLevel, Unnamed};
pub use spec::{
    ArchivedCap, ArchivedCapSlot, ArchivedCapTableEntry, ArchivedIrqEntry, ArchivedNamedObject,
    ArchivedObject, ArchivedObjectId, ArchivedRights, ArchivedSpec, AsidSlotEntry, Cap, CapSlot,
    CapTableEntry, IrqEntry, NamedObject, Object, ObjectId, PortableBadge, PortableCPtr,
    PortableWord, Rights, Spec, TryFromCapError, TryFromObjectError, UntypedCover, cap, object,
};

pub use frame_init::{FileContent, FileContentRange};

#[cfg(feature = "deflate")]
pub use frame_init::DeflatedBytesContent;

#[cfg(feature = "std")]
pub use when_std::{FillMap, FillMapBuilder, InputSpec};

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // //

#[cfg(feature = "deflate")]
pub type SpecForInitializer = Spec<Option<String>, DeflatedBytesContent, EmbeddedFrameOffset>;

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
