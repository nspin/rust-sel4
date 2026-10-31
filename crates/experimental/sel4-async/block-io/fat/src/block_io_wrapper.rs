//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::marker::PhantomData;

use aligned::{A1, Aligned};
use block_device_driver::{BlockDevice, blocks_to_slice, blocks_to_slice_mut};

use sel4_async_block_io::{
    BlockIO, ConstantBlockSize, Operation,
    access::{Access, Witness},
};

const BLOCK_SIZE: usize = 512;

pub struct BlockIOWrapper<T, A> {
    inner: T,
    _phantom: PhantomData<A>,
}

impl<T, A> BlockIOWrapper<T, A> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T: BlockIO<A, BlockSize = ConstantBlockSize<512>>, A: Access> BlockDevice<BLOCK_SIZE>
    for BlockIOWrapper<T, A>
{
    type Error = T::Error;

    type Align = A1;

    async fn read(
        &mut self,
        block_address: u32,
        data: &mut [Aligned<Self::Align, [u8; BLOCK_SIZE]>],
    ) -> Result<(), Self::Error> {
        self.inner
            .read_or_write_blocks(
                block_address.into(),
                Operation::Read {
                    buf: blocks_to_slice_mut(data),
                    witness: A::ReadWitness::TRY_WITNESS.unwrap(),
                },
            )
            .await
    }

    async fn write(
        &mut self,
        block_address: u32,
        data: &[Aligned<Self::Align, [u8; BLOCK_SIZE]>],
    ) -> Result<(), Self::Error> {
        self.inner
            .read_or_write_blocks(
                block_address.into(),
                Operation::Write {
                    buf: blocks_to_slice(data),
                    witness: A::WriteWitness::TRY_WITNESS.unwrap(),
                },
            )
            .await
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(self.inner.num_blocks() * u64::try_from(BLOCK_SIZE).unwrap())
    }
}
