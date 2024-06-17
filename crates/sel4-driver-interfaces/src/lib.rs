//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::fmt;
// use core::cell::BorrowMutError;

pub mod block;
pub mod net;
pub mod serial;
pub mod timer;

pub trait HandleInterrupt {
    fn handle_interrupt(&mut self);
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Shared<T>(pub T);

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum SharedRefCellError<E> {
    Contention,
    Other(E),
}

// impl<E> From<BorrowMutError> for SharedRefCellError<E> {
//     fn from(_: BorrowMutError) -> Self {
//         Self::Contention
//     }
// }

// impl<E> From<E> for SharedRefCellError<E> {
//     fn from(err: E) -> Self {
//         Self::Other(err)
//     }
// }

impl<E: fmt::Display> fmt::Display for SharedRefCellError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Contention => write!(f, "contention"),
            Self::Other(err) => err.fmt(f),
        }
    }
}
