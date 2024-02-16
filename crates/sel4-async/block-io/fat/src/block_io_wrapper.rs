//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;
use core::marker::PhantomData;

use futures::future;

use sel4_async_block_io::{
    access::{Access, Witness},
    ConcreteConstantBlockSize, BlockIO, Operation,
};

use embedded_fatfs::device::BlockDevice;

pub struct BlockIOWrapper<T, A, const N: usize> {
    inner: T,
    _phantom: PhantomData<(A, [(); N])>,
}

impl<T, A, const N: usize> BlockIOWrapper<T, A, N> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T: BlockIO<A>, A: Access, const N: usize> BlockDevice<N>
    for BlockIOWrapper<T, A, N>
where
    T::BlockSize: ConstantBlockSize,
    T::BlockSize::BYTES = N,
{
    // type Error = Infallible;

    // async fn read(
    //     &self,
    //     blocks: &mut [fat::Block],
    //     start_block_idx: fat::BlockIdx,
    //     _reason: &str,
    // ) -> Result<(), Self::Error> {
    //     future::join_all(blocks.iter_mut().enumerate().map(|(i, block)| async move {
    //         let block_idx = u64::from(start_block_idx.0)
    //             .checked_add(i.try_into().unwrap())
    //             .unwrap();
    //         self.inner
    //             .read_or_write_blocks(
    //                 block_idx,
    //                 Operation::Read {
    //                     buf: &mut block.contents,
    //                     witness: A::ReadWitness::TRY_WITNESS.unwrap(),
    //                 },
    //             )
    //             .await
    //     }))
    //     .await;
    //     Ok(())
    // }

    // async fn write(
    //     &self,
    //     blocks: &[fat::Block],
    //     start_block_idx: fat::BlockIdx,
    // ) -> Result<(), Self::Error> {
    //     future::join_all(blocks.iter().enumerate().map(|(i, block)| async move {
    //         let block_idx = u64::from(start_block_idx.0)
    //             .checked_add(i.try_into().unwrap())
    //             .unwrap();
    //         self.inner
    //             .read_or_write_blocks(
    //                 block_idx,
    //                 Operation::Write {
    //                     buf: &block.contents,
    //                     witness: A::WriteWitness::TRY_WITNESS.unwrap(),
    //                 },
    //             )
    //             .await
    //     }))
    //     .await;
    //     Ok(())
    // }

    // async fn num_blocks(&self) -> Result<fat::BlockCount, Self::Error> {
    //     Ok(fat::BlockCount(self.inner.num_blocks().try_into().unwrap()))
    // }
}
