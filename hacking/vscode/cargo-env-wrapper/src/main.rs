//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use cargo_metadata::{MetadataCommand, Package};
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

struct Env<'a> {
    cli: &'a Cli,
}

enum CargoTreeOutput {
    Packages(Vec<String>),
    InvalidFeatures(Vec<String>),
}

impl<'a> Env<'a> {
    fn run(&self) {
        let metadata = {
            let mut cmd = MetadataCommand::new();
            if let Some(s) = self.cli.manifest_path.as_ref() {
                cmd.manifest_path(s);
            }
            cmd.other_options(self.forward_features_args());
            cmd.no_deps();
            cmd.exec().unwrap()
        };

        for pkg in metadata.workspace_packages() {}
    }

    fn get_deps(&self, pkg: &Package) -> Vec<String> {
        todo!()
    }

    fn invoke_cargo_tree(&self, exclude_features: &[impl AsRef<str>]) -> CargoTreeOutput {
        let mut cmd = Command::new("cargo");
        cmd.arg("tree");
        if let Some(s) = self.cli.manifest_path.as_ref() {
            cmd.arg("--manifest-path").arg(s);
        }
        cmd.args(self.forward_args_with_feature_filter(|s| {
            !exclude_features.iter().any(|s_| s_.as_ref() == s)
        }));
        let output = cmd.output().unwrap();
        if output.status.success() {
            todo!()
        } else {
            todo!()
        }
    }

    fn forward_args(&self) -> Vec<String> {
        self.forward_args_with_feature_filter(|_| true)
    }

    fn forward_args_with_feature_filter(
        &self,
        feature_filter: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut args = self.forward_features_args_with_feature_filter(&feature_filter);
        if self.cli.all_features {
            args.push("--all-features".to_owned());
        }
        if self.cli.no_default_features {
            args.push("--no-default-features".to_owned());
        }
        if let Some(s) = self.cli.target.as_ref() {
            args.push("--target".to_owned());
            args.push(s.to_owned());
        }
        for s in self.cli.config.iter() {
            args.push("--config".to_owned());
            args.push(s.to_owned());
        }
        args
    }

    fn forward_features_args(&self) -> Vec<String> {
        self.forward_features_args_with_feature_filter(|_| true)
    }

    fn forward_features_args_with_feature_filter(
        &self,
        feature_filter: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut args = vec![];
        for s in self.cli.features.iter() {
            if let Some(filtered) = filter_features_arg(&feature_filter, s) {
                args.push("--features".to_owned());
                args.push(filtered.to_owned());
            }
        }
        if self.cli.all_features {
            args.push("--all-features".to_owned());
        }
        if self.cli.no_default_features {
            args.push("--no-default-features".to_owned());
        }
        args
    }
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

fn main() {
    let cli = Cli::parse();
    Env { cli: &cli }.run()
}
