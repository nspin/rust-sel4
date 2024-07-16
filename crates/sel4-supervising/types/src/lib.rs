//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::fmt;

use zerocopy::AsBytes;

#[cfg(target_pointer_width = "64")]
type Word = u64;

#[cfg(target_pointer_width = "32")]
type Word = u32;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum MemoryAccessWidth {
    U8,
    U16,
    U32,
    #[cfg(target_pointer_width = "64")]
    U64,
}

impl MemoryAccessWidth {
    pub const fn num_bytes(self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
            #[cfg(target_pointer_width = "64")]
            Self::U64 => 8,
        }
    }

    pub const fn truncate(self, val: Word) -> MemoryAccessData {
        match self {
            Self::U8 => MemoryAccessData::U8(val as u8),
            Self::U16 => MemoryAccessData::U16(val as u16),
            Self::U32 => MemoryAccessData::U32(val as u32),
            #[cfg(target_pointer_width = "64")]
            Self::U64 => MemoryAccessData::U64(val as u64),
        }
    }

    pub const fn mask(self) -> Word {
        self.truncate(!0).zero_extend()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MemoryAccessData {
    U8(u8),
    U16(u16),
    U32(u32),
    #[cfg(target_pointer_width = "64")]
    U64(u64),
}

impl MemoryAccessData {
    pub const fn width(&self) -> MemoryAccessWidth {
        match self {
            Self::U8(_) => MemoryAccessWidth::U8,
            Self::U16(_) => MemoryAccessWidth::U16,
            Self::U32(_) => MemoryAccessWidth::U32,
            #[cfg(target_pointer_width = "64")]
            Self::U64(_) => MemoryAccessWidth::U64,
        }
    }

    pub const fn zero_extend(&self) -> Word {
        match self {
            Self::U8(raw) => *raw as Word,
            Self::U16(raw) => *raw as Word,
            Self::U32(raw) => *raw as Word,
            #[cfg(target_pointer_width = "64")]
            Self::U64(raw) => *raw as Word,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::U8(raw) => raw.as_bytes(),
            Self::U16(raw) => raw.as_bytes(),
            Self::U32(raw) => raw.as_bytes(),
            #[cfg(target_pointer_width = "64")]
            Self::U64(raw) => raw.as_bytes(),
        }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        match self {
            Self::U8(raw) => raw.as_bytes_mut(),
            Self::U16(raw) => raw.as_bytes_mut(),
            Self::U32(raw) => raw.as_bytes_mut(),
            #[cfg(target_pointer_width = "64")]
            Self::U64(raw) => raw.as_bytes_mut(),
        }
    }

    pub const fn set(&self, old_val: Word) -> Word {
        (old_val & !self.width().mask()) | self.zero_extend()
    }
}

impl fmt::Display for MemoryAccessData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::U8(raw) => write!(f, "{:#04x}", raw),
            Self::U16(raw) => write!(f, "{:#06x}", raw),
            Self::U32(raw) => write!(f, "{:#010x}", raw),
            #[cfg(target_pointer_width = "64")]
            Self::U64(raw) => write!(f, "{:#018x}", raw),
        }
    }
}
