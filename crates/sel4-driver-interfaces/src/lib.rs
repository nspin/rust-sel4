//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub mod block;
pub mod net;
pub mod serial;
pub mod timer;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Shared<T>(pub T);

pub trait HandleInterrupt {
    fn handle_interrupt(&mut self);
}
