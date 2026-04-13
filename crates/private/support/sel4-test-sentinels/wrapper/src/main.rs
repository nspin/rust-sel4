//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::process::Command;

use anyhow::Error;

use sel4_test_sentinels_wrapper::*;

fn main() -> Result<(), Error> {
    let mut args = env::args_os();
    let _program_name = args.next();
    let child_program = args.next().expect("usage: wrapper <program> [args...]");
    let child_args = args.collect::<Vec<_>>();
    let mut cmd = Command::new(child_program);
    cmd.args(child_args);
    default_sentinels().wrap(cmd)?.map(|v| *v).success_ok()
}
