//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::io;
use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Error, bail};

pub struct Sentinels<T> {
    pub sequences: Vec<Sequence<T>>,
}

pub struct Sequence<T> {
    pub contiguous: bool,
    pub bytes: Vec<u8>,
    pub value: T,
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

pub fn default_sentinels() -> Sentinels<bool> {
    Sentinels {
        sequences: vec![
            Sequence {
                contiguous: false,
                bytes: b"INDICATE_SUCCESS\n\x06".to_vec(),
                value: true,
            },
            Sequence {
                contiguous: false,
                bytes: b"INDICATE_FAILURE\n\x15".to_vec(),
                value: true,
            },
            Sequence {
                contiguous: true,
                bytes: b"TEST_PASS".to_vec(),
                value: true,
            },
            Sequence {
                contiguous: true,
                bytes: b"TEST_FAIL".to_vec(),
                value: true,
            },
        ],
    }
}

#[derive(Debug)]
pub enum WrapperResult<T> {
    Sentinel(T),
    Exit(ExitStatus),
}

impl<T> WrapperResult<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> WrapperResult<U> {
        match self {
            Self::Sentinel(v) => WrapperResult::Sentinel(f(v)),
            Self::Exit(c) => WrapperResult::Exit(c)
        }
    }
}

impl WrapperResult<bool> {
    pub fn success_ok(&self) -> Result<(), Error> {
        match self {
            Self::Sentinel(false) => bail!("failure via sentinel"),
            Self::Exit(c) if !c.success() => bail!("failure via exit code (code: {})", c.code().map(|i| i.to_string()).unwrap_or("unknown".to_owned())),
            _ => Ok(()),
        }
    }
}

impl<T> Sentinels<T> {
    pub fn wrap(&self, mut cmd: Command) -> Result<WrapperResult<&T>, Error> {
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
}
