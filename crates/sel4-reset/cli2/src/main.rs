//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

use anyhow::{Error, ensure};
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

    let in_bytes = fs::read(&cli.in_file_path)?;
    let in_file = File::parse(in_bytes.as_slice())?;

    let out_bytes = match in_file {
        File::Elf32(elf) => continue_with_type(&elf),
        File::Elf64(elf) => continue_with_type(&elf),
        _ => {
            panic!()
        }
    }?;

    fs::write(cli.out_file_path, &out_bytes)?;

    todo!()
}

fn continue_with_type<'a, T>(orig_elf: &'a ElfFile<'a, T>) -> Result<Vec<u8>, Error>
where
    T: FileHeader<Word: NumCast>,
{
    Ok(vec![])
}
