//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::fmt;
use core::ops::Range;

#[cfg(feature = "deflate")]
use core::iter;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::object;

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FrameInit {
    Fill(Fill<DeflatedBytesContent>),
    Embedded(EmbeddedFrameOffset),
}

impl FrameInit {
    pub const fn as_fill(&self) -> Option<&Fill<DeflatedBytesContent>> {
        match self {
            Self::Fill(fill) => Some(fill),
            _ => None,
        }
    }

    pub const fn as_embedded(&self) -> Option<&EmbeddedFrameOffset> {
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

impl ArchivedFrameInit {
    pub const fn as_fill(&self) -> Option<&ArchivedFill<DeflatedBytesContent>> {
        match self {
            Self::Fill(fill) => Some(fill),
            _ => None,
        }
    }
}

impl<D> object::Frame<Fill<D>> {
    pub fn can_embed(&self, granule_size_bits: u8, is_root: bool) -> bool {
        is_root
            && self.paddr.is_none()
            && self.size_bits == granule_size_bits
            && !self.init.is_empty()
            && !self.init.depends_on_bootinfo()
    }
}

// // //

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct EmbeddedFrameOffset {
    pub offset: u64,
}

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Fill<D> {
    pub entries: Vec<FillEntry<D>>,
}

impl<D> Fill<D> {
    fn depends_on_bootinfo(&self) -> bool {
        self.entries.iter().any(|entry| entry.content.is_bootinfo())
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn traverse_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&Range<u64>, &D) -> Result<D1, E>,
    ) -> Result<Fill<D1>, E> {
        Ok(Fill {
            entries: self
                .entries
                .iter()
                .map(|entry| {
                    Ok(FillEntry {
                        range: entry.range.clone(),
                        content: match &entry.content {
                            FillEntryContent::BootInfo(content_bootinfo) => {
                                FillEntryContent::BootInfo(*content_bootinfo)
                            }
                            FillEntryContent::Data(content_data) => {
                                FillEntryContent::Data(f(&entry.range, content_data)?)
                            }
                        },
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    pub fn traverse<D1>(&self, mut f: impl FnMut(&Range<u64>, &D) -> D1) -> Fill<D1> {
        self.traverse_fallible(|x1, x2| Ok(f(x1, x2)))
            .unwrap_or_else(|absurdity: Infallible| match absurdity {})
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
