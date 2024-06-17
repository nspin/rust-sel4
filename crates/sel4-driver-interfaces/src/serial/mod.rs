//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Deref;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::{ErrorType, Read, Write};

use crate::Shared;

mod write_buffered;

pub use write_buffered::WriteBuffered;

impl<T: Deref<Target: ErrorType>> ErrorType for Shared<T> {
    type Error = T::Target::Error;
}
