//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::{RefCell, RefMut};
use core::fmt;
use core::ops::Deref;

pub mod block;
pub mod net;
pub mod serial;
pub mod timer;

pub trait HandleInterrupt {
    fn handle_interrupt(&mut self);
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct WrappedRefCell<T>(pub T);

impl<T> WrappedRefCell<T> {
    fn try_borrow_mut<E, U>(&self) -> Result<RefMut<U>, WrappedRefCellError<E>>
    where
        T: Deref<Target = RefCell<U>>,
    {
        self.0
            .deref()
            .try_borrow_mut()
            .map_err(|_| WrappedRefCellError::Contention)
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum WrappedRefCellError<E> {
    Contention,
    Other(E),
}

impl<E> From<E> for WrappedRefCellError<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}

impl<E: fmt::Display> fmt::Display for WrappedRefCellError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Contention => write!(f, "contention"),
            Self::Other(err) => err.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct WrappedMutex<T>(pub T);
