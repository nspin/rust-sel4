//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::convert::Infallible;
use core::marker::PhantomData;

use futures::future;

use embedded_fatfs::device::BlockDevice;

use sel4_async_block_io::{
    access::{Access, Witness},
    BlockIO, ConcreteConstantBlockSize, Operation,
};

pub use embedded_fatfs::*;

pub struct BlockIOAdapter<T, A, const N: usize> {
    inner: T,
    _phantom: PhantomData<(A, [(); N])>,
}

impl<T: Clone, A, const N: usize> Clone for BlockIOAdapter<T, A, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T, A, const N: usize> BlockIOAdapter<T, A, N> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T: BlockIO<A, BlockSize = ConcreteConstantBlockSize<N>>, A: Access, const N: usize>
    BlockDevice<N> for BlockIOAdapter<T, A, N>
{
    type Error = Infallible;

    async fn read(&mut self, block_address: u32, data: &mut [[u8; N]]) -> Result<(), Self::Error> {
        let shared = &self;
        future::join_all(data.iter_mut().enumerate().map(|(i, buf)| async move {
            shared
                .inner
                .read_or_write_blocks(
                    u64::from(block_address)
                        .checked_add(i.try_into().unwrap())
                        .unwrap(),
                    Operation::Read {
                        buf,
                        witness: A::ReadWitness::TRY_WITNESS.unwrap(),
                    },
                )
                .await
        }))
        .await;
        Ok(())
    }

    async fn write(&mut self, block_address: u32, data: &[[u8; N]]) -> Result<(), Self::Error> {
        let shared = &self;
        future::join_all(data.iter().enumerate().map(|(i, buf)| async move {
            shared
                .inner
                .read_or_write_blocks(
                    u64::from(block_address)
                        .checked_add(i.try_into().unwrap())
                        .unwrap(),
                    Operation::Write {
                        buf,
                        witness: A::WriteWitness::TRY_WITNESS.unwrap(),
                    },
                )
                .await
        }))
        .await;
        Ok(())
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(self.inner.num_blocks())
    }
}
