use alloc::boxed::Box;
use core::task::{Context as TaskContext, Poll};

use futures::future;

// TODO remove after bumping rust toolchain
use async_trait::async_trait;

pub trait AsyncIO {
    type Error;

    fn poll_read(
        &mut self,
        cx: &mut TaskContext<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_write(
        &mut self,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

#[derive(Copy, Clone, Debug)]
pub enum ClosedError<E> {
    Other(E),
    Closed,
}

impl<E> From<E> for ClosedError<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}

#[async_trait(?Send)]
pub trait AsyncIOExt: AsyncIO {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        future::poll_fn(|cx| self.poll_read(cx, buf)).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ClosedError<Self::Error>> {
        let mut pos = 0;
        while pos < buf.len() {
            let n = self.read(&mut buf[pos..]).await?;
            if n == 0 {
                return Err(ClosedError::Closed);
            }
            pos += n;
        }
        assert_eq!(pos, buf.len());
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        future::poll_fn(|cx| self.poll_write(cx, buf)).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), ClosedError<Self::Error>> {
        let mut pos = 0;
        while pos < buf.len() {
            let n = self.write(&buf[pos..]).await?;
            if n == 0 {
                return Err(ClosedError::Closed);
            }
            pos += n;
        }
        assert_eq!(pos, buf.len());
        Ok(())
    }
}

impl<T: AsyncIO + ?Sized> AsyncIOExt for T {}
