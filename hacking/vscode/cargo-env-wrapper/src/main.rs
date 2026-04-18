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

fn forward_args(cli: &Cli, feature_filter: impl Fn(&str) -> bool) -> Vec<String> {
    forward_args_with_feature_filter(cli, |_| true)
}

fn forward_args_with_feature_filter(
    cli: &Cli,
    feature_filter: impl Fn(&str) -> bool,
) -> Vec<String> {
    let mut args = vec![];
    for s in cli.features.iter() {
        if let Some(filtered) = filter_features_arg(&feature_filter, s) {
            args.push("--features".to_owned());
            args.push(filtered.to_owned());
        }
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

fn filter_features_arg(feature_filter: impl Fn(&str) -> bool, arg: &str) -> Option<String> {
    let filtered = arg
        .split(',')
        .inspect(|s| check_feature_arg_element(s))
        .filter(|s| feature_filter(s))
        .collect::<Vec<_>>()
        .join(",");
    if filtered.is_empty() {
        None
    } else {
        Some(filtered)
    }
}

fn check_feature_arg_element(s: &str) {
    assert_eq!(s.chars().filter(|c| *c == '/').count(), 1)
}
