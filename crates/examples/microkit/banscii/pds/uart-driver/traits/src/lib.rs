//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub trait UartDriver {
    unsafe fn new(ptr: *mut ()) -> Self;

    fn put_char(&self, c: u8);

    fn get_char(&self) -> Option<u8>;

    fn handle_interrupt(&self);
}
