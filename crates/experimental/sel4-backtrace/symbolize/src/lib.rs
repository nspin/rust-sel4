//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2016-2018 The gimli Developers
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

// Adapted from:
// https://github.com/gimli-rs/addr2line/blob/0.24.2/src/bin/addr2line.rs

#![no_std]

extern crate alloc;

#[cfg(feature = "loader")]
extern crate std;

use alloc::borrow::Cow;
use core::fmt;

use addr2line::gimli;
use addr2line::{Context, FrameIter, Location};

pub trait FindFrames {
    type Error: fmt::Debug;

    fn find_frames(
        &self,
        probe: u64,
    ) -> Result<FrameIter<'_, impl gimli::Reader + '_>, Self::Error>;

    fn find_location(&self, probe: u64) -> Result<Option<Location<'_>>, Self::Error>;

    fn find_symbol(&self, _probe: u64) -> Option<&str> {
        None
    }
}

impl<R: gimli::Reader> FindFrames for Context<R> {
    type Error = gimli::Error;

    fn find_frames(
        &self,
        probe: u64,
    ) -> Result<FrameIter<'_, impl gimli::Reader + '_>, Self::Error> {
        Context::find_frames(self, probe).skip_all_loads()
    }

    fn find_location(&self, probe: u64) -> Result<Option<Location<'_>>, Self::Error> {
        Context::find_location(self, probe)
    }
}

#[cfg(feature = "loader")]
impl FindFrames for addr2line::Loader {
    type Error = alloc::boxed::Box<dyn std::error::Error>;

    fn find_frames(
        &self,
        probe: u64,
    ) -> Result<FrameIter<'_, impl gimli::Reader + '_>, Self::Error> {
        addr2line::Loader::find_frames(self, probe)
    }

    fn find_location(&self, probe: u64) -> Result<Option<Location<'_>>, Self::Error> {
        addr2line::Loader::find_location(self, probe)
    }

    fn find_symbol(&self, probe: u64) -> Option<&str> {
        addr2line::Loader::find_symbol(self, probe)
    }
}

fn print_loc(w: &mut impl fmt::Write, loc: Option<&Location<'_>>) -> Result<(), fmt::Error> {
    if let Some(loc) = loc {
        if let Some(ref file) = loc.file.as_ref() {
            write!(w, "{file}:")?;
        } else {
            write!(w, "??:")?;
        }
        if let Some(line) = loc.line {
            write!(w, "{line}")?;
        } else {
            write!(w, "?")?;
        }
        writeln!(w)?;
    } else {
        writeln!(w, "??:0")?;
    }
    Ok(())
}

fn print_function(
    w: &mut impl fmt::Write,
    name: Option<&str>,
    language: Option<gimli::DwLang>,
    demangle: bool,
) -> Result<(), fmt::Error> {
    if let Some(name) = name {
        if demangle {
            write!(w, "{}", addr2line::demangle_auto(Cow::from(name), language))?;
        } else {
            write!(w, "{name}")?;
        }
    } else {
        write!(w, "??")?;
    }
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Options {
    pub do_functions: bool,
    pub do_inlines: bool,
    pub demangle: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            do_functions: true,
            do_inlines: true,
            demangle: true,
        }
    }
}

pub struct Symbolizer<'a, W, T> {
    w: &'a mut W,
    ctx: &'a T,
    opts: Options,
}

impl<'a, W: fmt::Write, T: FindFrames> Symbolizer<'a, W, T> {
    pub fn new(w: &'a mut W, ctx: &'a T, opts: Options) -> Self {
        Self { w, ctx, opts }
    }

    pub fn symbolize(&mut self, addr: u64) -> Result<(), fmt::Error> {
        write!(self.w, "{addr:#018x}  ")?;

        if self.opts.do_functions || self.opts.do_inlines {
            let mut printed_anything = false;
            let mut frames = self.ctx.find_frames(addr).unwrap();
            let mut first = true;
            while let Some(frame) = frames.next().unwrap() {
                if !first {
                    write!(self.w, "{: >18}  ", "(inlined by)")?;
                }
                first = false;

                if self.opts.do_functions {
                    // TODO
                    // See:
                    // https://github.com/gimli-rs/addr2line/blob/621a3abe985b32f43dd1e8c10e003abe902c68e2/src/bin/addr2line.rs#L223-L242
                    if let Some(func) = frame.function {
                        print_function(
                            self.w,
                            func.raw_name().ok().as_deref(),
                            func.language,
                            self.opts.demangle,
                        )?;
                    } else {
                        print_function(self.w, None, None, self.opts.demangle)?;
                    }

                    write!(self.w, "  at ")?;
                }

                print_loc(self.w, frame.location.as_ref())?;

                printed_anything = true;

                if !self.opts.do_inlines {
                    break;
                }
            }

            if !printed_anything {
                if self.opts.do_functions {
                    let name = self.ctx.find_symbol(addr);

                    print_function(self.w, name, None, self.opts.demangle)?;

                    write!(self.w, "  at ")?;
                }

                print_loc(self.w, None)?;
            }
        } else {
            let loc = self.ctx.find_location(addr).unwrap();
            print_loc(self.w, loc.as_ref())?;
        }

        Ok(())
    }
}
