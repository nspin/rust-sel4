//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use block_device_adapters::BufStream;
pub use embedded_fatfs::*;

mod block_io_wrapper;

pub use block_io_wrapper::BlockIOWrapper;
