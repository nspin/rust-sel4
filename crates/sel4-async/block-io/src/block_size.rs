//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::num::NonZero;

#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub trait BlockSize {
    fn bytes(&self) -> usize;
}

pub trait BlockType: BlockSize {
    type Block: AsRef<[u8]> + AsMut<[u8]>;

    fn zeroed_block(&self) -> Self::Block;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct ConstantBlockSize<const N: usize>;

impl<const N: usize> BlockSize for ConstantBlockSize<N> {
    fn bytes(&self) -> usize {
        N
    }
}

impl<const N: usize> BlockType for ConstantBlockSize<N> {
    type Block = [u8; N];

    fn zeroed_block(&self) -> Self::Block {
        [0; N]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RuntimeBlockSize {
    bytes: NonZero<usize>,
}

impl RuntimeBlockSize {
    pub const fn new(bytes: NonZero<usize>) -> Self {
        Self { bytes }
    }
}

impl BlockSize for RuntimeBlockSize {
    fn bytes(&self) -> usize {
        self.bytes.get()
    }
}

#[cfg(feature = "alloc")]
impl BlockType for RuntimeBlockSize {
    type Block = Vec<u8>;

    fn zeroed_block(&self) -> Self::Block {
        vec![0; self.bytes()]
    }
}
