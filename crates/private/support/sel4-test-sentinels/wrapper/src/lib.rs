//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::io;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use anyhow::{Error, bail};

struct Sentinels<T> {
    sequences: Vec<Sequence<T>>,
}

struct Sequence<T> {
    contiguous: bool,
    bytes: Vec<u8>,
    value: T,
}

struct Observer<T> {
    sentinels: Sentinels<T>,
    states: Vec<usize>,
}

impl<T> Observer<T> {
    fn new(sentinels: Sentinels<T>) -> Self {
        let n = sentinels.sequences.len();
        Self {
            sentinels,
            states: vec![0, n],
        }
    }

    fn observe(&mut self, b: u8) -> Option<&T> {
        for (sequence, i) in self.sentinels.sequences.iter_mut().zip(self.states.iter_mut()) {
            if b == sequence.bytes[*i] {
                *i += 1;
                if *i == sequence.bytes.len() {
                    return Some(&sequence.value)
                }
            } else if !sequence.contiguous {
                *i = 0;
            }
        }
        None
    }    
}

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
