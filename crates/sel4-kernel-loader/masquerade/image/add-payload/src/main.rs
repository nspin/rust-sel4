//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use clap::Parser;
use object::elf::{FileHeader32, FileHeader64};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endianness, ReadRef};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    loader: String,
    #[arg(long)]
    payload: String,
    #[arg(long, short = 'o')]
    out_file: String,
    #[arg(long, short = 'v')]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{cli:#?}");
    }

    Ok(())
}
