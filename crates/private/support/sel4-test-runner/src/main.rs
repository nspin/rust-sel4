//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::{env, fs};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio, exit};

use anyhow::Error;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    exe: PathBuf,
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

fn main() -> Result<ExitCode, Error> {
    let cli = Cli::parse();
    println!("{:?}", cli);

    let parent = cli.target_dir.join("runner");
    fs::create_dir_all(&parent)?;
    let mut d = TempDir::with_prefix_in("run-", parent)?;
    d.disable_cleanup(true);

    eprintln!("tmp:");
    eprintln!("{}", d.path().display());

    let exe = d.path().join(cli.exe.file_name().unwrap());
    fs::copy(&cli.exe, &exe)?;

    Ok(ExitCode::SUCCESS)
}
