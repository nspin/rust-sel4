//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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

pub struct RuntimeBlockSize {
    bits: usize,
}

impl RuntimeBlockSize {
    pub fn new(bits: usize) -> Self {
        Self { bits }
    }
}

impl BlockSize for RuntimeBlockSize {
    fn bytes(&self) -> usize {
        1 << self.bits
    }
}

#[cfg(feature = "alloc")]
impl BlockType for RuntimeBlockSize {
    type Block = Vec<u8>;

    fn zeroed_block(&self) -> Self::Block {
        vec![0; self.bytes()]
    }
}
