//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;

use anyhow::Error;

fn main() -> Result<(), Error> {
    let mut args = env::args_os();
    let _program_name = args.next();
    let child_program = args.next().expect("usage: wrapper <program> [args...]");
    let child_args = args.collect::<Vec<_>>();
    sel4_test_sentinels_wrapper::run(child_program, child_args)?.success_ok()
}
