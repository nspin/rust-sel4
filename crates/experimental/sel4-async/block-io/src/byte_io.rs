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

use crate::Operation;
use crate::access::{Access, ReadAccess, ReadOnly, ReadWrite, WriteAccess};

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
