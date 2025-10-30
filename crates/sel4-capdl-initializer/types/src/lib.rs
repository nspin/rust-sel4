//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::ops::Range;

use rkyv::primitive::{ArchivedU16, ArchivedU32, ArchivedU64};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod cap_table;
mod footprint;
mod frame_init;
mod inspect;
mod object_name;
mod spec;

mod traverse;

#[cfg(feature = "std")]
mod when_std;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{HasCapTable, PageTableEntry};
pub use footprint::Footprint;
pub use frame_init::{
    BytesContent, Content, EmbeddedFrame, Fill, FillEntry, FillEntryContent,
    FillEntryContentBootInfo, FillEntryContentBootInfoId, FrameInit, GetEmbeddedFrame,
    IndirectBytesContent, IndirectEmbeddedFrame, NeverEmbedded, SelfContainedContent,
    SelfContainedGetEmbeddedFrame,
};
pub use object_name::{
    IndirectObjectName, ObjectName, ObjectNamesLevel, SelfContainedObjectName, Unnamed,
};
pub use spec::{
    AsidSlotEntry, Cap, CapSlot, CapTableEntry, IrqEntry, NamedObject, Object, ObjectId,
    PortableBadge, PortableCPtr, PortableWord, Rights, Spec, TryFromCapError, TryFromObjectError,
    UntypedCover, cap, object,
};

pub use frame_init::{FileContent, FileContentRange};

#[cfg(feature = "deflate")]
pub use frame_init::{DeflatedBytesContent, IndirectDeflatedBytesContent};

#[cfg(feature = "std")]
pub use when_std::{FillMap, FillMapBuilder, InputSpec};

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpecWithSources<'a, N: ObjectName, D: Content, M: GetEmbeddedFrame> {
    pub spec: Spec<N, D, M>,
    pub object_name_source: &'a N::Source,
    pub content_source: &'a D::Source,
    pub embedded_frame_source: &'a M::Source,
}

#[cfg(feature = "deflate")]
pub type SpecWithIndirection =
    Spec<Option<IndirectObjectName>, IndirectDeflatedBytesContent, IndirectEmbeddedFrame>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct SelfContained<T>(T);

impl<T> SelfContained<T> {
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }

    pub const fn inner(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

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
