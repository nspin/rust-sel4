//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::marker::PhantomData;

use embedded_io_async::{self as eio, ErrorKind, Read, Seek, SeekFrom, Write};
use thiserror::Error;

use crate::{ByteIO, ByteIOLayout, ReadAccess, WriteAccess};

#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
pub enum EmbeddedIOAsyncAdapterError<E> {
    #[error("I/O error: {0:?}")]
    Io(E),
    #[error("invalid seek")]
    InvalidSeek,
    #[error("write past end")]
    WriteZero,
}

impl<E> From<E> for EmbeddedIOAsyncAdapterError<E> {
    fn from(err: E) -> Self {
        Self::Io(err)
    }
}

impl<E: fmt::Debug> eio::Error for EmbeddedIOAsyncAdapterError<E> {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Io(_) => ErrorKind::Other,
            Self::InvalidSeek => ErrorKind::InvalidInput,
            Self::WriteZero => ErrorKind::WriteZero,
        }
    }
}

pub struct EmbeddedIOAsyncAdapter<T, A> {
    inner: T,
    position: u64,
    _phantom: PhantomData<A>,
}

impl<T, A> EmbeddedIOAsyncAdapter<T, A> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            position: 0,
            _phantom: PhantomData,
        }
    }

    pub fn position(&self) -> u64 {
        self.position
    }

    wrapper_methods!(T);
}

impl<T: ByteIOLayout, A> EmbeddedIOAsyncAdapter<T, A> {
    fn remaining_for(&self, buf_len: usize) -> usize {
        let remaining = self.inner().size().saturating_sub(self.position);
        usize::try_from(remaining).map_or(buf_len, |remaining| remaining.min(buf_len))
    }
}

impl<T: ByteIOLayout, A> eio::ErrorType for EmbeddedIOAsyncAdapter<T, A> {
    type Error = EmbeddedIOAsyncAdapterError<T::Error>;
}

impl<T: ByteIO<A>, A: ReadAccess> Read for EmbeddedIOAsyncAdapter<T, A> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let n = self.remaining_for(buf.len());
        if n > 0 {
            self.inner().read(self.position, &mut buf[..n]).await?;
            self.position += u64::try_from(n).unwrap();
        }
        Ok(n)
    }
}

impl<T: ByteIO<A>, A: WriteAccess> Write for EmbeddedIOAsyncAdapter<T, A> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if buf.is_empty() {
            return Ok(0);
        }
        let n = self.remaining_for(buf.len());
        if n == 0 {
            return Err(EmbeddedIOAsyncAdapterError::WriteZero);
        }
        self.inner().write(self.position, &buf[..n]).await?;
        self.position += u64::try_from(n).unwrap();
        Ok(n)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T: ByteIOLayout, A> Seek for EmbeddedIOAsyncAdapter<T, A> {
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let (base, offset) = match pos {
            SeekFrom::Start(x) => (x, 0),
            SeekFrom::End(x) => (self.inner().size(), x),
            SeekFrom::Current(x) => (self.position, x),
        };
        self.position = base
            .checked_add_signed(offset)
            .ok_or(EmbeddedIOAsyncAdapterError::InvalidSeek)?;
        Ok(self.position)
    }
}
