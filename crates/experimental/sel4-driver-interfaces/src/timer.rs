//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::time::Duration;

pub trait ErrorType {
    type Error: fmt::Debug;
}

pub trait Clock: ErrorType {
    fn get_time(&mut self) -> Result<Duration, Self::Error>;
}

pub trait Timer: Clock {
    fn set_timeout(&mut self, relative: Duration) -> Result<(), Self::Error>;

    fn clear_timeout(&mut self) -> Result<(), Self::Error>;
}
