//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#[cfg(feature = "alloc")]
use alloc::rc::Rc;
use core::cell::RefCell;
use core::convert::Infallible;
use core::fmt;

use crate::access::{Access, ReadAccess, ReadOnly, ReadWrite, WriteAccess};
use futures::future;

use crate::{
    BlockIO, BlockIOLayout, BlockSize, CastAccessAdapter, Operation, PartialBlockIO,
    block_size_bytes_u64,
};

pub trait ByteIOLayout {
    type Error: fmt::Debug;

    fn size(&self) -> u64;
}

pub trait ByteIO<A: Access>: ByteIOLayout {
    #[allow(async_fn_in_trait)]
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error>;

    #[allow(async_fn_in_trait)]
    async fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error>
    where
        A: ReadAccess,
    {
        self.read_or_write(offset, Operation::read(buf)).await
    }

    #[allow(async_fn_in_trait)]
    async fn write(&self, offset: u64, buf: &[u8]) -> Result<(), Self::Error>
    where
        A: WriteAccess,
    {
        self.read_or_write(offset, Operation::write(buf)).await
    }
}

#[cfg(feature = "alloc")]
impl<T: ByteIOLayout> ByteIOLayout for Rc<T> {
    type Error = T::Error;

    fn size(&self) -> u64 {
        T::size(self)
    }
}

#[cfg(feature = "alloc")]
impl<T: ByteIO<A>, A: Access> ByteIO<A> for Rc<T> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        T::read_or_write(self, offset, operation).await
    }
}

pub struct SliceByteIO<T> {
    inner: T,
}

impl<T> SliceByteIO<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: AsRef<[u8]>> ByteIOLayout for SliceByteIO<T> {
    type Error = Infallible;

    fn size(&self) -> u64 {
        self.inner().as_ref().len().try_into().unwrap()
    }
}

impl<T: AsRef<[u8]>> ByteIO<ReadOnly> for SliceByteIO<T> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, ReadOnly>,
    ) -> Result<(), Self::Error> {
        let offset = offset.try_into().unwrap();
        let Operation::Read { buf, .. } = operation;
        buf.copy_from_slice(&self.inner().as_ref()[offset..][..buf.len()]);
        Ok(())
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ByteIOLayout for RefCell<SliceByteIO<T>> {
    type Error = Infallible;

    fn size(&self) -> u64 {
        self.borrow().size()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ByteIO<ReadWrite> for RefCell<SliceByteIO<T>> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, ReadWrite>,
    ) -> Result<(), Self::Error> {
        let offset = offset.try_into().unwrap();
        match operation {
            Operation::Read { buf, .. } => {
                buf.copy_from_slice(&self.borrow().inner().as_ref()[offset..][..buf.len()]);
            }
            Operation::Write { buf, .. } => {
                self.borrow_mut().inner_mut().as_mut()[offset..][..buf.len()].copy_from_slice(buf);
            }
        }
        Ok(())
    }
}

impl<T: ByteIOLayout, A> ByteIOLayout for CastAccessAdapter<T, A> {
    type Error = T::Error;

    fn size(&self) -> u64 {
        self.inner().size()
    }
}

impl<T: ByteIO<A>, A: Access, A1: Access> ByteIO<A1> for CastAccessAdapter<T, A> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, A1>,
    ) -> Result<(), Self::Error> {
        self.inner()
            .read_or_write(offset, operation.cast_access())
            .await
    }
}

#[derive(Clone, Debug)]
pub struct ByteIOAdapter<T> {
    inner: T,
}

impl<T> ByteIOAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: BlockIOLayout> ByteIOLayout for ByteIOAdapter<T> {
    type Error = T::Error;

    fn size(&self) -> u64 {
        self.inner().num_blocks() * block_size_bytes_u64(&self.inner().block_size())
    }
}

impl<T: PartialBlockIO<A>, A: Access> ByteIO<A> for ByteIOAdapter<T> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        read_or_write_bytes(self.inner(), offset, operation).await
    }
}

#[derive(Clone, Debug)]
pub struct BlockIOAdapter<T, N> {
    inner: T,
    block_size: N,
}

impl<T, N> BlockIOAdapter<T, N> {
    pub fn new(inner: T, block_size: N) -> Self {
        Self { inner, block_size }
    }

    wrapper_methods!(T);
}

impl<T: ByteIOLayout, N: BlockSize + Copy> BlockIOLayout for BlockIOAdapter<T, N> {
    type Error = T::Error;

    type BlockSize = N;

    fn block_size(&self) -> Self::BlockSize {
        self.block_size
    }

    fn num_blocks(&self) -> u64 {
        self.inner().size() / block_size_bytes_u64(&self.block_size())
    }
}

impl<T: ByteIO<A>, A: Access, N: BlockSize + Copy> BlockIO<A> for BlockIOAdapter<T, N> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        let start_byte_idx = start_block_idx
            .checked_mul(block_size_bytes_u64(&self.block_size()))
            .unwrap();
        self.inner().read_or_write(start_byte_idx, operation).await
    }
}

impl<T: ByteIO<A>, A: Access, N: BlockSize + Copy> PartialBlockIO<A> for BlockIOAdapter<T, N> {
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        assert!(offset + operation.len() <= self.block_size().bytes());
        let byte_idx = block_idx
            .checked_mul(block_size_bytes_u64(&self.block_size()))
            .and_then(|x| x.checked_add(offset.try_into().unwrap()))
            .unwrap();
        self.inner().read_or_write(byte_idx, operation).await
    }
}

async fn read_or_write_bytes<T: PartialBlockIO<A>, A: Access>(
    io: &T,
    offset: u64,
    mut operation: Operation<'_, A>,
) -> Result<(), T::Error> {
    let block_size = io.block_size().bytes();
    let block_size_u64 = block_size_bytes_u64(&io.block_size());
    let start_block_idx = offset / block_size_u64;
    let offset_into_start_block = usize::try_from(offset % block_size_u64).unwrap();

    let head_len = if offset_into_start_block == 0 {
        0
    } else {
        (block_size - offset_into_start_block).min(operation.len())
    };
    let (head, mut rest) = operation.split_at(head_len);
    let num_full_blocks = rest.len() / block_size;
    let (full_blocks, tail) = rest.split_at(num_full_blocks * block_size);
    let full_blocks_start_block_idx = start_block_idx + u64::from(head_len != 0);
    let tail_block_idx = full_blocks_start_block_idx + u64::try_from(num_full_blocks).unwrap();

    future::try_join3(
        async {
            if !head.is_empty() {
                io.read_or_write_block_at_offset(start_block_idx, offset_into_start_block, head)
                    .await?;
            }
            Ok(())
        },
        async {
            if !full_blocks.is_empty() {
                io.read_or_write_blocks(full_blocks_start_block_idx, full_blocks)
                    .await?;
            }
            Ok(())
        },
        async {
            if !tail.is_empty() {
                io.read_or_write_block_at_offset(tail_block_idx, 0, tail)
                    .await?;
            }
            Ok(())
        },
    )
    .await?;
    Ok(())
}

pub async fn read_bytes<T: PartialBlockIO<A>, A: ReadAccess>(
    io: &T,
    offset: u64,
    buf: &mut [u8],
) -> Result<(), T::Error> {
    read_or_write_bytes(io, offset, Operation::read(buf)).await
}

pub async fn write_bytes<T: PartialBlockIO<A>, A: WriteAccess>(
    io: &T,
    offset: u64,
    buf: &[u8],
) -> Result<(), T::Error> {
    read_or_write_bytes(io, offset, Operation::write(buf)).await
}
