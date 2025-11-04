//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;
use clap::Parser;
use serde_json::Value;
use similar::TextDiff;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    manifest_path: PathBuf,

    #[arg(short = 'i')]
    in_: PathBuf,

    #[arg(short = 'o')]
    out: PathBuf,

    #[arg(long = "just-check")]
    just_check: bool,
}

fn main() {
    let args = Cli::parse();

    let metadata = MetadataCommand::new()
        .manifest_path(&args.manifest_path)
        .no_deps()
        .exec()
        .unwrap();

    let default_members = metadata
        .workspace_default_members
        .iter()
        .collect::<HashSet<_>>();

    let mut obj = parse_file(&args.in_);

    let extra_args = obj
        .as_object_mut()
        .unwrap()
        .entry("rust-analyzer.cargo.extraArgs")
        .or_insert(Value::Array(vec![]));
    for pkg in metadata.workspace_packages() {
        if !default_members.contains(&pkg.id) {
            extra_args.as_array_mut().unwrap().append(&mut vec![
                Value::String("--exclude".to_owned()),
                Value::String(pkg.name.to_owned()),
            ]);
        }
    }

    if args.just_check {
        let existing_obj = parse_file(&args.out);
        if obj != existing_obj {
            let pretty_obj = serde_json::to_string_pretty(&obj).unwrap();
            let pretty_existing_obj = serde_json::to_string_pretty(&existing_obj).unwrap();
            panic!(
                "mismatch:\n{}",
                TextDiff::from_lines(&pretty_obj, &pretty_existing_obj).unified_diff(),
            );
        }
    } else {
        fs::write(args.out, serde_json::to_string_pretty(&obj).unwrap()).unwrap();
    }
}

fn parse_file(path: impl AsRef<Path>) -> Value {
    serde_json5::from_str::<Value>(&fs::read_to_string(path).unwrap()).unwrap()
}
