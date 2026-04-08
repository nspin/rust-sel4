//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::io::{Read, Write};
use std::process::{Command, ExitCode, Stdio, exit};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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
    microkit_board: String,
    #[arg(long)]
    microkit_config: String,
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> ExitCode {
    ExitCode::SUCCESS
}
