//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::{env, fs, io, iter};

use anyhow::Error;
use clap::Parser;
use object::{Architecture, File, Object, ObjectSection as _};
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

#[derive(Debug)]
enum SeL4TestKind {
    RootTask,
    Microkit,
    CapDL,
}

fn get_sel4_test_kind(file: &File) -> Option<SeL4TestKind> {
    if file.symbol_by_name("sel4_test_kind_root_task").is_some() {
        Some(SeL4TestKind::RootTask)
    } else if file.symbol_by_name("sel4_test_kind_microkit").is_some() {
        Some(SeL4TestKind::Microkit)
    } else if file.symbol_by_name("sel4_test_kind_capdl").is_some() {
        Some(SeL4TestKind::CapDL)
    } else {
        None
    }
}

fn get_qemu_exe(file: &File) -> String {
    let qemu_arch = match file.architecture() {
        Architecture::Aarch64 => "aarch64",
        Architecture::Arm => "arm",
        Architecture::X86_64 => "x86_64",
        Architecture::X86_64_X32 => "i386",
        Architecture::Riscv32 => "riscv32",
        Architecture::Riscv64 => "riscv64",
        _ => todo!(),
    };
    format!("qemu-{qemu_arch}")
}

fn main() -> Result<(), Error> {
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

    match get_sel4_test_kind(&file) {
        None => {
            assert!(
                Command::new(get_qemu_exe(&file))
                    .args(
                        iter::once(exe.as_os_str())
                            .chain(cli.simulate_args.iter().map(AsRef::as_ref)),
                    )
                    .status()?
                    .success()
            );
        }
        Some(kind) => {
            let image = match kind {
                SeL4TestKind::RootTask => {
                    if let Architecture::X86_64 = file.architecture() {
                        exe.clone()
                    } else {
                        let image = d.path().join("image.elf");

                        let loader_target_config = ".cargo/gen/target/aarch64-unknown-none.toml";

                        assert!(
                            Command::new("cargo")
                                .arg("build")
                                .arg("--config")
                                .arg(loader_target_config)
                                .arg("--target-dir")
                                .arg(&cli.target_dir)
                                .arg("-p")
                                .arg("sel4-kernel-loader")
                                .arg("--artifact-dir")
                                .arg(d.path())
                                .status()?
                                .success()
                        );

                        assert!(
                            Command::new("cargo")
                                .arg("run")
                                .arg("-p")
                                .arg("sel4-kernel-loader-add-payload")
                                .arg("--")
                                .arg("--loader")
                                .arg(d.path().join("sel4-kernel-loader"))
                                .arg("--sel4-prefix")
                                .arg(env::var("SEL4_PREFIX").unwrap())
                                .arg("--app")
                                .arg(&exe)
                                .arg("-o")
                                .arg(&image)
                                .status()?
                                .success()
                        );

                        image
                    }
                }
                SeL4TestKind::Microkit => {
                    let system_xml = d.path().join("system.xml");
                    if let Some(sec) = file.section_by_name(".sdf_xml") {
                        fs::write(&system_xml, sec.data()?)?;
                    } else if let Some(sec) = file.section_by_name(".sdf_script") {
                        let system_py = d.path().join("system.py");
                        fs::write(&system_py, sec.data()?)?;
                        assert!(
                            Command::new("python3")
                                .arg(&system_py)
                                .arg("--board")
                                .arg(cli.microkit_board.as_ref().unwrap())
                                .arg("-o")
                                .arg(&system_xml)
                                .status()?
                                .success()
                        );
                    } else {
                        panic!("missing sdf")
                    }

                    let image = d.path().join("image.elf");

                    assert!(
                        Command::new(cli.microkit_tool.as_ref().unwrap())
                            .arg(&system_xml)
                            .arg("--search-path")
                            .arg(d.path())
                            .arg("--board")
                            .arg(cli.microkit_board.as_ref().unwrap())
                            .arg("--config")
                            .arg(cli.microkit_config.as_ref().unwrap())
                            .arg("-o")
                            .arg(&image)
                            .arg("-r")
                            .arg(d.path().join("report.txt"))
                            .status()?
                            .success()
                    );

                    image
                }
                SeL4TestKind::CapDL => {
                    todo!()
                }
            };

            let outcome = run(
                &cli.simulate_script,
                iter::once(image.as_os_str()).chain(cli.simulate_args.iter().map(AsRef::as_ref)),
            )?;

            assert!(Command::new("stty").arg("echo").status()?.success());

            match outcome {
                RunOutcome::Sentinel(success) => assert!(success),
                RunOutcome::Exit(success) => assert!(success),
            }
        }
    }

    Ok(())
}

const SUCCESS: u8 = 0x06;
const FAILURE: u8 = 0x15;

// TODO make sure text has passed first

#[derive(Debug)]
enum RunOutcome {
    Sentinel(bool),
    Exit(bool),
}

fn run(
    child_program: impl AsRef<OsStr>,
    child_args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<RunOutcome, Error> {
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
                    Some(true)
                } else if b == FAILURE {
                    Some(false)
                } else {
                    stdout.write_all(&buf)?;
                    stdout.flush()?;
                    None
                };

                if let Some(success) = exit_code_opt {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(RunOutcome::Sentinel(success));
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

    Ok(RunOutcome::Exit(child.wait()?.success()))
}
