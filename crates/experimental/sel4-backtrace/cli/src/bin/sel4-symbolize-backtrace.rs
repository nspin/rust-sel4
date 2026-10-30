//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use addr2line::Loader;
use clap::Parser;

use sel4_backtrace_types::Backtrace;

#[derive(Parser, Debug)]
struct Cli {
    raw_backtrace: String,
    #[arg(long, short = 'f')]
    file: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let bt_hex = &cli.raw_backtrace;
    let bt = Backtrace::<Option<String>>::recv(&hex::decode(bt_hex).unwrap()).unwrap();
    let elf_file_path = cli
        .file
        .as_ref()
        .or(bt.preamble.image.as_ref())
        .expect("ELF file neither embedded nor provided");
    let loader = Loader::new(elf_file_path).unwrap();
    println!("backtrace: {elf_file_path}");
    let mut s = String::new();
    bt.symbolize(&loader, &mut s).unwrap();
    print!("{s}");
}
