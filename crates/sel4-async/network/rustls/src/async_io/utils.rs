use alloc::vec::Vec;

use rustls::unbuffered::InsufficientSizeError;

use super::Error;

pub(crate) struct WriteCursor<'a> {
    buf: &'a mut [u8],
    used: usize,
}

impl<'a> WriteCursor<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, used: 0 }
    }

    // TODO new
    pub(crate) fn used(&self) -> usize {
        self.used
    }

    pub(crate) fn into_used(self) -> usize {
        self.used
    }

    pub(crate) fn append<'b>(&mut self, data: &'b [u8]) -> &'b [u8] {
        let len = self.remaining_capacity().min(data.len());

        self.unfilled()[..len].copy_from_slice(&data[..len]);
        self.used += len;

        data.split_at(len).1
    }

    pub(crate) fn unfilled(&mut self) -> &mut [u8] {
        &mut self.buf[self.used..]
    }

    pub(crate) fn is_full(&self) -> bool {
        self.remaining_capacity() == 0
    }

    pub(crate) fn remaining_capacity(&self) -> usize {
        self.buf.len() - self.used
    }
}

#[derive(Default)]
pub(crate) struct Buffer {
    inner: Vec<u8>,
    used: usize,
}

impl Buffer {
    pub(crate) fn advance(&mut self, num_bytes: usize) {
        self.used += num_bytes;
    }

    pub(crate) fn discard(&mut self, num_bytes: usize) {
        if num_bytes == 0 {
            return;
        }

        debug_assert!(num_bytes <= self.used);

        self.inner.copy_within(num_bytes..self.used, 0);
        self.used -= num_bytes;

        log::trace!("discarded {num_bytes}B");
    }

    pub(crate) fn reserve(&mut self, additional_bytes: usize) {
        let new_len = self.used + additional_bytes;
        self.inner.resize(new_len, 0);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn len(&self) -> usize {
        self.filled().len()
    }

    pub(crate) fn filled(&self) -> &[u8] {
        &self.inner[..self.used]
    }

    pub(crate) fn filled_mut(&mut self) -> &mut [u8] {
        &mut self.inner[..self.used]
    }

    pub(crate) fn unfilled(&mut self) -> &mut [u8] {
        &mut self.inner[self.used..]
    }

    pub(crate) fn capacity(&self) -> usize {
        self.inner.len()
    }
}

pub(crate) fn try_or_resize_and_retry<E1, E2>(
    mut f: impl FnMut(&mut [u8]) -> Result<usize, E1>,
    map_err: impl FnOnce(E1) -> Result<InsufficientSizeError, Error<E2>>,
    outgoing: &mut Buffer,
) -> Result<usize, Error<E2>>
where
    Error<E2>: From<E1>,
{
    let written = match f(outgoing.unfilled()) {
        Ok(written) => written,

        Err(e) => {
            let InsufficientSizeError { required_size } = map_err(e)?;
            outgoing.reserve(required_size);
            log::trace!("resized `outgoing_tls` buffer to {}B", outgoing.capacity());

            f(outgoing.unfilled())?
        }
    };

    outgoing.advance(written);

    Ok(written)
}