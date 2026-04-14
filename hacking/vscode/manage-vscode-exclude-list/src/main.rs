//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;
use clap::Parser;
use similar::TextDiff;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    manifest_path: PathBuf,

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
        .collect::<BTreeSet<_>>();

    let mut excludes = BTreeSet::new();
    for pkg in metadata.workspace_packages() {
        if !default_members.contains(&pkg.id) {
            excludes.insert(pkg.name.as_str());
        }
    }
    let excludes_str = excludes_to_string(&excludes);

    if args.just_check {
        let existing_excludes_input_str = fs::read_to_string(&args.out).unwrap();
        let existing_excludes = existing_excludes_input_str.lines().collect::<BTreeSet<_>>();
        if excludes != existing_excludes {
            let existing_excludes_str = excludes_to_string(&existing_excludes);
            panic!(
                "mismatch:\n{}",
                TextDiff::from_lines(&excludes_str, &existing_excludes_str).unified_diff(),
            );
        }
    } else {
        fs::write(args.out, excludes_str).unwrap();
    }
}

fn excludes_to_string(excludes: &BTreeSet<&str>) -> String {
    let mut s = String::new();
    for pkg_name in excludes {
        s.push_str(pkg_name);
        s.push('\n');
    }
    s
}
