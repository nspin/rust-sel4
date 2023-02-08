use core::fmt;
use core::ops::Range;

#[cfg(feature = "deflate")]
use core::iter;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::SelfContained;

// // //

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileContent {
    pub file: String,
    pub file_offset: usize,
}

// // //

pub trait SelfContainedContent {
    fn self_contained_copy_out(&self, dst: &mut [u8]);
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BytesContent<'a> {
    pub bytes: &'a [u8],
}

#[cfg(feature = "alloc")]
impl<'a> BytesContent<'a> {
    pub fn pack(raw_content: &[u8]) -> Vec<u8> {
        raw_content.to_vec()
    }
}

impl<'a> SelfContainedContent for BytesContent<'a> {
    fn self_contained_copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(self.bytes)
    }
}

impl<'a> fmt::Debug for BytesContent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytesContent")
            .field("bytes", &"&[...]")
            .finish()
    }
}

#[cfg(feature = "deflate")]
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeflatedBytesContent<'a> {
    pub deflated_bytes: &'a [u8],
}

#[cfg(all(feature = "alloc", feature = "deflate"))]
impl<'a> DeflatedBytesContent<'a> {
    pub fn pack(raw_content: &[u8]) -> Vec<u8> {
        miniz_oxide::deflate::compress_to_vec(raw_content, 10)
    }
}

#[cfg(feature = "deflate")]
impl<'a> SelfContainedContent for DeflatedBytesContent<'a> {
    fn self_contained_copy_out(&self, dst: &mut [u8]) {
        let n = miniz_oxide::inflate::decompress_slice_iter_to_slice(
            dst,
            iter::once(self.deflated_bytes),
            false, // zlib_header
            true,  // ignore_adler32
        )
        .unwrap();
        assert_eq!(n, dst.len())
    }
}

#[cfg(feature = "deflate")]
impl<'a> fmt::Debug for DeflatedBytesContent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeflatedBytesContent")
            .field("deflated_bytes", &"&[...]")
            .finish()
    }
}

// // //

pub trait Content {
    type Source: ?Sized;

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]);
}

impl<T: SelfContainedContent> Content for SelfContained<T> {
    type Source = ();

    fn copy_out(&self, _source: &Self::Source, dst: &mut [u8]) {
        self.inner().self_contained_copy_out(dst)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndirectBytesContent {
    pub bytes_range: Range<usize>,
}

impl Content for IndirectBytesContent {
    type Source = [u8];

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]) {
        BytesContent {
            bytes: &source[self.bytes_range.clone()],
        }
        .self_contained_copy_out(dst)
    }
}

#[cfg(feature = "deflate")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndirectDeflatedBytesContent {
    pub deflated_bytes_range: Range<usize>,
}

#[cfg(feature = "deflate")]
impl Content for IndirectDeflatedBytesContent {
    type Source = [u8];

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]) {
        DeflatedBytesContent {
            deflated_bytes: &source[self.deflated_bytes_range.clone()],
        }
        .self_contained_copy_out(dst)
    }
}
