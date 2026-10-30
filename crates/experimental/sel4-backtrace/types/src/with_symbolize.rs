//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

#![allow(clippy::write_with_newline)]

use core::fmt;

use addr2line::gimli::Error;

use sel4_backtrace_symbolize::{FindFrames, symbolize};

use crate::Backtrace;

impl<T> Backtrace<T> {
    pub fn symbolize(&self, ctx: &impl FindFrames, w: &mut impl fmt::Write) -> Result<(), Error> {
        if let Some(ref err) = self.postamble.error {
            writeln!(w, "    error: {err:?}").unwrap();
        }
        symbolize(
            w,
            ctx,
            &Default::default(),
            self.entries
                .iter()
                .map(|entry| entry.stack_frame.ip.try_into().unwrap()),
        )
        .unwrap();
        Ok(())
    }
}
