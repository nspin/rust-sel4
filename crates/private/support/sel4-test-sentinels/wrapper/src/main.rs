//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::io::{Read, Write};
use std::process::{Command, Stdio, exit};

const SUCCESS: u8 = 0x06;
const FAILURE: u8 = 0x15;

fn main() -> std::io::Result<()> {
    let mut args = env::args_os();
    let _program_name = args.next();

    let Some(child_program) = args.next() else {
        eprintln!("usage: wrapper <program> [args...]");
        exit(2);
    };

    let child_args = args.collect::<Vec<_>>();

    let mut child = Command::new(child_program)
        .args(&child_args)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_stdout = child.stdout.take().unwrap();

    let mut stdout = std::io::stdout().lock();

    loop {
        let mut buf = [0u8; 1];

        match child_stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(1) => {
                let b = buf[0];

                stdout.write_all(&buf)?;
                stdout.flush()?;

                let exit_code_opt = if b == SUCCESS {
                    Some(0)
                } else if b == FAILURE {
                    Some(1)
                } else {
                    None
                };

                if let Some(code) = exit_code_opt {
                    let _ = child.kill();
                    let _ = child.wait();
                    exit(code);
                }
            }
            Ok(_) => unreachable!(),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        }
    }

    match child.wait()?.code() {
        Some(code) => exit(code),
        None => exit(1),
    }
}
