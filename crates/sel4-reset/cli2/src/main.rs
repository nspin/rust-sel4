//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

use anyhow::{Error, ensure};
use anyhow::Result;
use num::NumCast;
use object::elf::{PF_W, PT_LOAD};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endian, File, Object, ObjectSection, ReadCache, ReadRef};
use rangemap::RangeSet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    in_file_path: PathBuf,
    #[arg(short = 'o')]
    out_file_path: PathBuf,
    #[arg(long, short = 'v')]
    interactive: bool,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    todo!()
}
