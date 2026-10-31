//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

#[cfg(feature = "alloc")]
use alloc::rc::Rc;

use futures::future;

pub mod access;

mod operation;

pub use operation::{CastAccessError, NoReadAccess, NoWriteAccess, Operation, OperationType};

use access::{Access, ReadAccess, ReadOnly, WriteAccess};

macro_rules! wrapper_methods {
    ($inner:path) => {
        pub fn into_inner(self) -> $inner {
            self.inner
        }

        pub const fn inner(&self) -> &$inner {
            &self.inner
        }

        pub fn inner_mut(&mut self) -> &mut $inner {
            &mut self.inner
        }
    };
}

mod block_size;

pub use block_size::{BlockSize, BlockType, ConstantBlockSize, RuntimeBlockSize};

mod byte_io;

pub use byte_io::{ByteIO, ByteIOLayout, SliceByteIO};

#[cfg(feature = "embedded-io-async")]
mod when_embedded_io_async;

#[cfg(feature = "embedded-io-async")]
pub use when_embedded_io_async::{EmbeddedIOAsyncAdapter, EmbeddedIOAsyncAdapterError};

pub trait BlockIOLayout {
    type Error: fmt::Debug;

    type BlockSize: BlockSize;

    fn block_size(&self) -> Self::BlockSize;

    fn num_blocks(&self) -> u64;
}

pub trait BlockIO<A: Access>: BlockIOLayout {
    #[allow(async_fn_in_trait)]
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error>;

    #[allow(async_fn_in_trait)]
    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error>
    where
        A: ReadAccess,
    {
        self.read_or_write_blocks(
            start_block_idx,
            Operation::Read {
                buf,
                witness: A::READ_WITNESS,
            },
        )
        .await
    }

    #[allow(async_fn_in_trait)]
    async fn write_blocks(&self, start_block_idx: u64, buf: &[u8]) -> Result<(), Self::Error>
    where
        A: WriteAccess,
    {
        self.read_or_write_blocks(
            start_block_idx,
            Operation::Write {
                buf,
                witness: A::WRITE_WITNESS,
            },
        )
        .await
    }
}

impl<T: BlockIOLayout> BlockIOLayout for &T {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        T::block_size(self)
    }

    fn num_blocks(&self) -> u64 {
        T::num_blocks(self)
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for &T {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), <&T as BlockIOLayout>::Error> {
        T::read_or_write_blocks(self, start_block_idx, operation).await
    }
}

#[cfg(feature = "alloc")]
impl<T: BlockIOLayout> BlockIOLayout for Rc<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        T::block_size(self)
    }

    fn num_blocks(&self) -> u64 {
        T::num_blocks(self)
    }
}

#[cfg(feature = "alloc")]
impl<T: BlockIO<A>, A: Access> BlockIO<A> for Rc<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        T::read_or_write_blocks(self, start_block_idx, operation).await
    }
}

#[derive(Clone, Debug)]
pub struct Partition<T> {
    inner: T,
    range: Range<u64>,
}

impl<T: BlockIO<ReadOnly>> Partition<T> {
    pub fn new(inner: T, range: Range<u64>) -> Self {
        assert!(range.start <= range.end);
        assert!(range.end <= inner.num_blocks());
        Self { inner, range }
    }
}

impl<T> Partition<T> {
    wrapper_methods!(T);
}

impl<T: BlockIOLayout> BlockIOLayout for Partition<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.inner().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.range.end - self.range.start
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for Partition<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        assert!(
            start_block_idx
                + usize_to_u64(operation.len()) / usize_to_u64(self.block_size().bytes())
                <= self.num_blocks()
        );
        let inner_block_idx = self.range.start + start_block_idx;
        self.inner()
            .read_or_write_blocks(inner_block_idx, operation)
            .await
    }
}

#[derive(Clone, Debug)]
pub struct CastAccessAdapter<T, A> {
    inner: T,
    _phantom: PhantomData<A>,
}

impl<T, A> CastAccessAdapter<T, A> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    wrapper_methods!(T);
}

impl<T: BlockIOLayout, A> BlockIOLayout for CastAccessAdapter<T, A> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.inner().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.inner().num_blocks()
    }
}

impl<T: BlockIO<A>, A: Access, A1: Access> BlockIO<A1> for CastAccessAdapter<T, A> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A1>,
    ) -> Result<(), Self::Error> {
        self.inner()
            .read_or_write_blocks(start_block_idx, operation.cast_access())
            .await
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
        self.inner().num_blocks() * usize_to_u64(self.inner().block_size().bytes())
    }
}

impl<T: BlockIO<A, BlockSize: BlockType>, A: ReadAccess> ByteIO<A> for ByteIOAdapter<T> {
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
        self.inner().size() / usize_to_u64(self.block_size().bytes())
    }
}

impl<T: ByteIO<A>, A: Access, N: BlockSize + Copy> BlockIO<A> for BlockIOAdapter<T, N> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        let start_byte_idx = start_block_idx
            .checked_mul(usize_to_u64(self.block_size().bytes()))
            .unwrap();
        self.inner().read_or_write(start_byte_idx, operation).await
    }
}

async fn read_or_write_partial_block<T: BlockIO<A, BlockSize: BlockType>, A: ReadAccess>(
    io: &T,
    block_idx: u64,
    offset_into_block: usize,
    operation: Operation<'_, A>,
) -> Result<(), T::Error> {
    assert!(offset_into_block + operation.len() <= io.block_size().bytes());
    let mut block_buf = io.block_size().zeroed_block();
    io.read_blocks(block_idx, block_buf.as_mut()).await?;
    match operation {
        Operation::Read { buf, .. } => {
            buf.copy_from_slice(&block_buf.as_ref()[offset_into_block..][..buf.len()]);
        }
        Operation::Write { buf, witness } => {
            block_buf.as_mut()[offset_into_block..][..buf.len()].copy_from_slice(buf);
            io.read_or_write_blocks(
                block_idx,
                Operation::Write {
                    buf: block_buf.as_ref(),
                    witness,
                },
            )
            .await?;
        }
    }
    Ok(())
}

async fn read_or_write_bytes<T: BlockIO<A, BlockSize: BlockType>, A: ReadAccess>(
    io: &T,
    offset: u64,
    mut operation: Operation<'_, A>,
) -> Result<(), T::Error> {
    let block_size = io.block_size().bytes();
    let block_size_u64 = usize_to_u64(block_size);
    let byte_offset_of_first_full_block = offset.next_multiple_of(block_size_u64);
    let byte_offset_of_first_full_block_in_buf =
        usize::try_from(byte_offset_of_first_full_block - offset).unwrap();
    let first_full_block_idx = byte_offset_of_first_full_block / block_size_u64;
    let num_full_blocks = (operation.len() - byte_offset_of_first_full_block_in_buf) / block_size;
    if byte_offset_of_first_full_block > offset + usize_to_u64(operation.len()) {
        let block_idx = first_full_block_idx - 1;
        let offset_into_block = offset - block_idx * block_size_u64;
        read_or_write_partial_block(
            io,
            block_idx,
            offset_into_block.try_into().unwrap(),
            operation,
        )
        .await?;
    } else {
        let (left_partial_block, mut rest) =
            operation.split_at(byte_offset_of_first_full_block_in_buf);
        let (full_blocks, right_partial_block) = rest.split_at(num_full_blocks * block_size);
        future::try_join3(
            async {
                io.read_or_write_blocks(first_full_block_idx, full_blocks)
                    .await
            },
            async {
                if !left_partial_block.is_empty() {
                    let block_idx = first_full_block_idx - 1;
                    let offset_into_block = block_size - left_partial_block.len();
                    read_or_write_partial_block(
                        io,
                        block_idx,
                        offset_into_block,
                        left_partial_block,
                    )
                    .await?;
                }
                Ok(())
            },
            async {
                if !right_partial_block.is_empty() {
                    let block_idx = first_full_block_idx + usize_to_u64(num_full_blocks);
                    let offset_into_block = 0;
                    read_or_write_partial_block(
                        io,
                        block_idx,
                        offset_into_block,
                        right_partial_block,
                    )
                    .await?;
                }
                Ok(())
            },
        )
        .await?;
    }
    Ok(())
}

pub async fn read_bytes<T: BlockIO<A, BlockSize: BlockType>, A: ReadAccess>(
    io: &T,
    offset: u64,
    buf: &mut [u8],
) -> Result<(), T::Error> {
    read_or_write_bytes(io, offset, Operation::read(buf)).await
}

pub async fn write_bytes<T: BlockIO<A, BlockSize: BlockType>, A: ReadAccess + WriteAccess>(
    io: &T,
    offset: u64,
    buf: &[u8],
) -> Result<(), T::Error> {
    read_or_write_bytes(io, offset, Operation::write(buf)).await
}

fn usize_to_u64(n: usize) -> u64 {
    n.try_into().unwrap()
}
