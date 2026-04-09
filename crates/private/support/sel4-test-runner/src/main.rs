//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::io::{Read, Write};
use std::process::{Command, ExitCode, Stdio, exit};

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    target_dir: PathBuf,
    #[arg(long)]
    object_sizes: Option<PathBuf>,
    #[arg(long)]
    sel4_kernel_config: Option<PathBuf>,
    #[arg(long)]
    simulate_script: PathBuf,
    #[arg(long)]
    microkit_tool: Option<PathBuf>,
    #[arg(long)]
    microkit_board: Option<String>,
    #[arg(long)]
    microkit_config: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli);
}
