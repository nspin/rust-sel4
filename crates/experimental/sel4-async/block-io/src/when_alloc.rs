//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Deref;

use crate::{Access, BlockIO, BlockIOLayout, BlockSize, Operation};

pub struct DynamicBlockSize {
    bits: usize,
}

impl DynamicBlockSize {
    pub fn new(bits: usize) -> Self {
        Self { bits }
    }
}

impl BlockSize for DynamicBlockSize {
    type Block = Vec<u8>;

    fn bytes(&self) -> usize {
        1 << self.bits
    }

    fn zeroed_block(&self) -> Self::Block {
        vec![0; self.bytes()]
    }
}

impl<T: BlockIOLayout> BlockIOLayout for Rc<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.deref().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.deref().num_blocks()
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for Rc<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        self.deref()
            .read_or_write_blocks(start_block_idx, operation)
            .await
    }
}
