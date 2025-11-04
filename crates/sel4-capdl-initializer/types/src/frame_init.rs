//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::{string::String, vec::Vec};
use core::fmt;
use core::ops::Range;
use serde::{Deserializer, Serializer};

#[cfg(feature = "deflate")]
use core::iter;

use rkyv::Archive;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::object;

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FrameInit<D, M> {
    Fill(Fill<D>),
    Embedded(M),
}

impl<D, M> FrameInit<D, M> {
    pub const fn as_fill(&self) -> Option<&Fill<D>> {
        match self {
            Self::Fill(fill) => Some(fill),
            _ => None,
        }
    }

    pub const fn as_embedded(&self) -> Option<&M> {
        match self {
            Self::Embedded(embedded) => Some(embedded),
            _ => None,
        }
    }

    pub const fn is_fill(&self) -> bool {
        self.as_fill().is_some()
    }

    pub const fn is_embedded(&self) -> bool {
        self.as_embedded().is_some()
    }
}

impl<D> FrameInit<D, NeverEmbedded> {
    #[allow(clippy::explicit_auto_deref)]
    pub const fn as_fill_infallible(&self) -> &Fill<D> {
        match self {
            Self::Fill(fill) => fill,
            Self::Embedded(absurdity) => match *absurdity {},
        }
    }
}

impl<D: Archive, M: Archive> ArchivedFrameInit<D, M> {
    pub const fn as_fill(&self) -> Option<&ArchivedFill<D>> {
        match self {
            Self::Fill(fill) => Some(fill),
            _ => None,
        }
    }
}

impl<D> object::Frame<D, NeverEmbedded> {
    pub fn can_embed(&self, granule_size_bits: u8, is_root: bool) -> bool {
        is_root
            && self.paddr.is_none()
            && self.size_bits == granule_size_bits
            && !self.init.as_fill_infallible().is_empty()
            && !self.init.as_fill_infallible().depends_on_bootinfo()
    }
}

// // //

#[derive(Copy, Clone)]
pub enum NeverEmbedded {}

impl Serialize for NeverEmbedded {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {}
    }
}

impl<'de> Deserialize<'de> for NeverEmbedded {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "cannot deserialize `NeverEmbedded`",
        ))
    }
}

// // //

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct EmbeddedFrameOffset {
    offset: u64,
}

impl EmbeddedFrameOffset {
    pub const fn new(offset: u64) -> Self {
        Self { offset }
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }
}

unsafe impl Sync for EmbeddedFrameOffset {}

pub trait GetEmbeddedFrameOffset {
    fn get_embedded_frame(&self) -> EmbeddedFrameOffset;
}

impl GetEmbeddedFrameOffset for EmbeddedFrameOffset {
    fn get_embedded_frame(&self) -> EmbeddedFrameOffset {
        *self
    }
}

impl GetEmbeddedFrameOffset for ArchivedEmbeddedFrameOffset {
    fn get_embedded_frame(&self) -> EmbeddedFrameOffset {
        EmbeddedFrameOffset {
            offset: self.offset.into(),
        }
    }
}

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Fill<D> {
    pub entries: Vec<FillEntry<D>>,
}

impl<D> Fill<D> {
    pub fn depends_on_bootinfo(&self) -> bool {
        self.entries.iter().any(|entry| entry.content.is_bootinfo())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FillEntry<D> {
    pub range: Range<u64>,
    pub content: FillEntryContent<D>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FillEntryContent<D> {
    Data(D),
    BootInfo(FillEntryContentBootInfo),
}

impl<D> FillEntryContent<D> {
    pub fn as_data(&self) -> Option<&D> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_bootinfo(&self) -> Option<&FillEntryContentBootInfo> {
        match self {
            Self::BootInfo(info) => Some(info),
            _ => None,
        }
    }

    pub fn is_data(&self) -> bool {
        self.as_data().is_some()
    }

    pub fn is_bootinfo(&self) -> bool {
        self.as_bootinfo().is_some()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FillEntryContentBootInfo {
    pub id: FillEntryContentBootInfoId,
    pub offset: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FillEntryContentBootInfoId {
    Fdt,
}

// // //

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FileContent {
    pub file: String,
    pub file_offset: u64,
}

// // //

pub trait Content {
    fn copy_out(&self, dst: &mut [u8]);
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct BytesContent {
    pub bytes: Vec<u8>,
}

impl BytesContent {
    pub fn pack(raw_content: &[u8]) -> Self {
        Self {
            bytes: raw_content.to_vec(),
        }
    }
}

impl fmt::Debug for BytesContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytesContent")
            .field("bytes", &"&[...]")
            .finish()
    }
}

impl Content for BytesContent {
    fn copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.bytes)
    }
}

impl Content for ArchivedBytesContent {
    fn copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.bytes)
    }
}

#[cfg(feature = "deflate")]
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct DeflatedBytesContent {
    pub deflated_bytes: Vec<u8>,
}

#[cfg(feature = "deflate")]
impl DeflatedBytesContent {
    pub fn pack(raw_content: &[u8]) -> Self {
        Self {
            deflated_bytes: miniz_oxide::deflate::compress_to_vec(raw_content, 10),
        }
    }
}

#[cfg(feature = "deflate")]
impl fmt::Debug for DeflatedBytesContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeflatedBytesContent")
            .field("deflated_bytes", &"&[...]")
            .finish()
    }
}

#[cfg(feature = "deflate")]
impl Content for DeflatedBytesContent {
    fn copy_out(&self, dst: &mut [u8]) {
        copy_out_deflated(&self.deflated_bytes, dst)
    }
}

#[cfg(feature = "deflate")]
impl Content for ArchivedDeflatedBytesContent {
    fn copy_out(&self, dst: &mut [u8]) {
        copy_out_deflated(&self.deflated_bytes, dst)
    }
}

#[cfg(feature = "deflate")]
fn copy_out_deflated(deflated_src: &[u8], dst: &mut [u8]) {
    let n = miniz_oxide::inflate::decompress_slice_iter_to_slice(
        dst,
        iter::once(deflated_src),
        false, // zlib_header
        true,  // ignore_adler32
    )
    .unwrap();
    assert_eq!(n, dst.len())
}
