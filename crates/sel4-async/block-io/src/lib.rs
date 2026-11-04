//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::rc::Rc;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

pub mod access;

mod operation;

pub use operation::{CastAccessError, NoReadAccess, NoWriteAccess, Operation, OperationType};

use access::{Access, ReadAccess, WriteAccess};

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

pub use byte_io::{
    BlockIOAdapter, ByteIO, ByteIOAdapter, ByteIOLayout, SliceByteIO, read_bytes, write_bytes,
};

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

    /// Completes once every write which completed before this call is durable.
    ///
    /// The default implementation does nothing, which is correct for implementations whose writes
    /// are durable once they complete.
    #[allow(async_fn_in_trait)]
    async fn flush(&self) -> Result<(), Self::Error> {
        Ok(())
    }

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

pub trait PartialBlockIO<A: Access>: BlockIO<A> {
    /// Reads or writes `operation.len()` bytes, starting `offset` bytes into block `block_idx`.
    ///
    /// `offset + operation.len()` must not exceed the block size.
    #[allow(async_fn_in_trait)]
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error>;

    #[allow(async_fn_in_trait)]
    async fn read_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<(), Self::Error>
    where
        A: ReadAccess,
    {
        self.read_or_write_block_at_offset(block_idx, offset, Operation::read(buf))
            .await
    }

    #[allow(async_fn_in_trait)]
    async fn write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        buf: &[u8],
    ) -> Result<(), Self::Error>
    where
        A: WriteAccess,
    {
        self.read_or_write_block_at_offset(block_idx, offset, Operation::write(buf))
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

    async fn flush(&self) -> Result<(), <&T as BlockIOLayout>::Error> {
        T::flush(self).await
    }
}

impl<T: PartialBlockIO<A>, A: Access> PartialBlockIO<A> for &T {
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        T::read_or_write_block_at_offset(self, block_idx, offset, operation).await
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

    async fn flush(&self) -> Result<(), Self::Error> {
        T::flush(self).await
    }
}

#[cfg(feature = "alloc")]
impl<T: PartialBlockIO<A>, A: Access> PartialBlockIO<A> for Rc<T> {
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        T::read_or_write_block_at_offset(self, block_idx, offset, operation).await
    }
}

#[derive(Clone, Debug)]
pub struct Partition<T> {
    inner: T,
    range: Range<u64>,
}

impl<T: BlockIOLayout> Partition<T> {
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
                + u64::try_from(operation.len()).unwrap()
                    / block_size_bytes_u64(&self.block_size())
                <= self.num_blocks()
        );
        let inner_block_idx = self.range.start + start_block_idx;
        self.inner()
            .read_or_write_blocks(inner_block_idx, operation)
            .await
    }

    async fn flush(&self) -> Result<(), Self::Error> {
        self.inner().flush().await
    }
}

impl<T: PartialBlockIO<A>, A: Access> PartialBlockIO<A> for Partition<T> {
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        assert!(block_idx < self.num_blocks());
        let inner_block_idx = self.range.start + block_idx;
        self.inner()
            .read_or_write_block_at_offset(inner_block_idx, offset, operation)
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

    async fn flush(&self) -> Result<(), Self::Error> {
        self.inner().flush().await
    }
}

impl<T: PartialBlockIO<A>, A: Access, A1: Access> PartialBlockIO<A1> for CastAccessAdapter<T, A> {
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A1>,
    ) -> Result<(), Self::Error> {
        self.inner()
            .read_or_write_block_at_offset(block_idx, offset, operation.cast_access())
            .await
    }
}

#[derive(Clone, Debug)]
pub struct InefficientPartialBlockIO<T> {
    inner: T,
}

impl<T> InefficientPartialBlockIO<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: BlockIOLayout> BlockIOLayout for InefficientPartialBlockIO<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.inner().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.inner().num_blocks()
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for InefficientPartialBlockIO<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        self.inner()
            .read_or_write_blocks(start_block_idx, operation)
            .await
    }

    async fn flush(&self) -> Result<(), Self::Error> {
        self.inner().flush().await
    }
}

impl<T: BlockIO<A, BlockSize: BlockType>, A: ReadAccess> PartialBlockIO<A>
    for InefficientPartialBlockIO<T>
{
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        let io = self.inner();
        assert!(offset + operation.len() <= io.block_size().bytes());
        let mut block_buf = io.block_size().zeroed_block();
        io.read_blocks(block_idx, block_buf.as_mut()).await?;
        match operation {
            Operation::Read { buf, .. } => {
                buf.copy_from_slice(&block_buf.as_ref()[offset..][..buf.len()]);
            }
            Operation::Write { buf, witness } => {
                block_buf.as_mut()[offset..][..buf.len()].copy_from_slice(buf);
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
}

fn block_size_bytes_u64(block_size: &impl BlockSize) -> u64 {
    block_size.bytes().try_into().unwrap()
}
