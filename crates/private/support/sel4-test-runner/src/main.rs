//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio, exit};
use std::{env, fs, io, iter};

use anyhow::{Error, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use object::{Architecture, Object};
use tempfile::TempDir;

mod wrapper;

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
    #[arg(last = true)]
    simulate_args: Option<String>,
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

    let data = fs::read(&exe)?;
    let file = object::File::parse(&*data)?;

    let image = if let Architecture::X86_64 = file.architecture() {
        exe.clone()
    } else {
        let image = d.path().join("image.elf");

        image
    };

    let code = run(
        &cli.simulate_script,
        iter::once(image.as_os_str()).chain(cli.simulate_args.iter().map(AsRef::as_ref)),
    )?;

    assert!(Command::new("stty").arg("echo").status()?.success());

    Ok(code)
}

const SUCCESS: u8 = 0x06;
const FAILURE: u8 = 0x15;

// TODO make sure text has passed first

fn run(
    child_program: impl AsRef<OsStr>,
    child_args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<ExitCode, Error> {
    let mut child = Command::new(child_program.as_ref())
        .args(child_args)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_stdout = child.stdout.take().unwrap();

    let mut stdout = io::stdout().lock();

    loop {
        let mut buf = [0u8; 1];

        match child_stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(1) => {
                let b = buf[0];

                let exit_code_opt = if b == SUCCESS {
                    Some(ExitCode::SUCCESS)
                } else if b == FAILURE {
                    Some(ExitCode::FAILURE)
                } else {
                    stdout.write_all(&buf)?;
                    stdout.flush()?;
                    None
                };

                if let Some(code) = exit_code_opt {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(code);
                }
            }
            Ok(_) => unreachable!(),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.into());
            }
        }
    }

    Ok(if child.wait()?.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
