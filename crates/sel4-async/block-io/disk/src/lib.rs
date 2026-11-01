//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub mod mbr;

pub struct Disk<T> {
    io: T,
}

impl<T> Disk<T> {
    pub fn new(io: T) -> Self {
        Self { io }
    }
}
