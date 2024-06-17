//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;
use core::ops::Deref;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::{Error, ErrorKind, ErrorType, Read, Write};

use crate::{Shared, SharedRefCellError};

mod write_buffered;

pub use write_buffered::WriteBuffered;

impl<E: Error> Error for SharedRefCellError<E> {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Contention => ErrorKind::Other,
            Self::Other(err) => err.kind(),
        }
    }
}

impl<T: Deref<Target = RefCell<U>>, U: ErrorType> ErrorType for Shared<T> {
    type Error = SharedRefCellError<<U as ErrorType>::Error>;
}

impl<Word: Copy, T: Deref<Target = RefCell<U>>, U: Read<Word>> Read<Word> for Shared<T> {
    fn read(&mut self) -> nb::Result<Word, Self::Error> {
        self.0
            .deref()
            .try_borrow_mut()
            .map_err(|_| SharedRefCellError::Contention)?
            .read()
            .map_err(|err| err.map(SharedRefCellError::Other))
    }
}

impl<Word: Copy, T: Deref<Target = RefCell<U>>, U: Write<Word>> Write<Word> for Shared<T> {
    fn write(&mut self, word: Word) -> nb::Result<(), Self::Error> {
        self.0
            .deref()
            .try_borrow_mut()
            .map_err(|_| SharedRefCellError::Contention)?
            .write(word)
            .map_err(|err| err.map(SharedRefCellError::Other))
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.0
            .deref()
            .try_borrow_mut()
            .map_err(|_| SharedRefCellError::Contention)?
            .flush()
            .map_err(|err| err.map(SharedRefCellError::Other))
    }
}
