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

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    manifest_path: Option<PathBuf>,

    #[arg(long, short = 'F')]
    features: Vec<String>,

    #[arg(long)]
    all_features: bool,

    #[arg(long)]
    no_default_features: bool,

    #[arg(long)]
    target: Option<String>,

    #[arg(long)]
    config: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
}

fn forward_args(cli: &Cli) -> Vec<String> {
    let mut args = vec![];
    for s in cli.features.iter() {
        args.push("--features".to_owned());
        args.push(s.to_owned());
    }
    if cli.all_features {
        args.push("--all-features".to_owned());
    }
    if cli.no_default_features {
        args.push("--all-features".to_owned());
    }
    if let Some(s) = cli.target.as_ref() {
        args.push("--target".to_owned());
        args.push(s.to_owned());
    }
    for s in cli.config.iter() {
        args.push("--config".to_owned());
        args.push(s.to_owned());
    }
    args
}
