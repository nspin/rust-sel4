//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::io;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use anyhow::{Error, bail};

const SUCCESS: u8 = 0x06;
const FAILURE: u8 = 0x15;

// TODO make sure text has passed first

#[derive(Debug)]
pub enum SentinelsOutcome {
    Sentinel(bool),
    Exit(bool),
}

impl SentinelsOutcome {
    pub fn success_ok(&self) -> Result<(), Error> {
        match self {
            Self::Sentinel(false) => bail!("failure via sentinel"),
            Self::Exit(false) => bail!("failure via exit code"),
            _ => Ok(()),
        }
    }
}

pub fn run(mut cmd: Command) -> Result<SentinelsOutcome, Error> {
    let mut child = cmd.stdin(Stdio::null()).stdout(Stdio::piped()).spawn()?;

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
                    return Ok(SentinelsOutcome::Sentinel(success));
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

    Ok(SentinelsOutcome::Exit(child.wait()?.success()))
}
