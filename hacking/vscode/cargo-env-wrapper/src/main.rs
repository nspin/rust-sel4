//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;

use cargo_metadata::{MetadataCommand, PackageName};
use clap::Parser;

fn main() {
    Env::get().run()
}

#[derive(Debug, Parser)]
struct Cli {
    wrapped: String,

    #[arg(last = true)]
    wrapped_args: Option<String>,

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

    #[arg(long, short = 'e')]
    exclude: Vec<String>,
}

struct Env {
    cli: Cli,
}

enum CargoTreeOutput {
    Packages(Vec<String>),
    InvalidFeatures(Vec<String>),
}

impl Env {
    fn get() -> Self {
        let cli = Cli::parse();
        Self { cli }
    }

    fn run(&self) {
        for pkg in self.excludes().iter() {
            println!("{pkg}");
        }
    }

    fn excludes(&self) -> BTreeSet<PackageName> {
        let metadata = {
            let mut cmd = MetadataCommand::new();
            if let Some(s) = self.cli.manifest_path.as_ref() {
                cmd.manifest_path(s);
            }
            cmd.other_options(self.forward_features_args());
            cmd.no_deps();
            cmd.exec().unwrap()
        };

        let workspace_pkgs = metadata
            .workspace_packages()
            .iter()
            .map(|pkg| &pkg.name)
            .collect::<BTreeSet<_>>();

        let mut pkg_names = BTreeMap::new();
        for pkg in workspace_pkgs.iter() {
            pkg_names.insert(pkg.as_ref(), pkg);
        }

        let exclude = self
            .cli
            .exclude
            .iter()
            .map(|name| pkg_names[name.as_str()])
            .collect::<BTreeSet<_>>();

        let fast_ok = self
            .get_fast_ok()
            .iter()
            .map(|name| pkg_names[name.as_str()])
            .collect::<BTreeSet<_>>();

        let mut ok = BTreeSet::new();
        for pkg in workspace_pkgs.iter() {
            let is_ok = if fast_ok.contains(pkg) {
                true
            } else {
                let raw_deps = self.get_deps(pkg);
                let deps = raw_deps
                    .iter()
                    .map(|name| pkg_names[name.as_str()])
                    .collect::<BTreeSet<_>>();
                deps.intersection(&exclude).count() == 0
            };
            if is_ok {
                ok.insert(*pkg);
            }
        }

        BTreeSet::from_iter(workspace_pkgs.difference(&ok).map(|pkg| (*pkg).clone()))
    }

    fn get_fast_ok(&self) -> Vec<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("tree");
        if let Some(s) = self.cli.manifest_path.as_ref() {
            cmd.arg("--manifest-path").arg(s);
        }
        cmd.arg("--workspace");
        cmd.arg("--prefix").arg("none").arg("--format").arg("{p}");
        cmd.arg("--color").arg("never");
        for pkg in self.cli.exclude.iter() {
            cmd.arg("--invert").arg(pkg);
        }
        let output = cmd.output().unwrap();
        assert!(output.status.success());
        str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .filter_map(|s| {
                if s.contains(" (/") {
                    Some(s.split_whitespace().next().unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    fn get_deps(&self, pkg: &PackageName) -> Vec<String> {
        match self.invoke_cargo_tree::<&str>(pkg, &[]) {
            CargoTreeOutput::Packages(pkgs) => pkgs,
            CargoTreeOutput::InvalidFeatures(feats) => match self.invoke_cargo_tree(pkg, &feats) {
                CargoTreeOutput::Packages(pkgs) => pkgs,
                _ => panic!(),
            },
        }
    }

    fn invoke_cargo_tree<T: AsRef<str>>(
        &self,
        pkg: &PackageName,
        exclude_features: &[T],
    ) -> CargoTreeOutput {
        let mut cmd = Command::new("cargo");
        cmd.arg("tree");
        if let Some(s) = self.cli.manifest_path.as_ref() {
            cmd.arg("--manifest-path").arg(s);
        }
        cmd.arg("--package").arg(pkg.as_ref());
        cmd.args(self.forward_args_with_feature_filter(|s| {
            !exclude_features.iter().any(|s_| s_.as_ref() == s)
        }));
        cmd.arg("--prefix").arg("none").arg("--format").arg("{p}");
        cmd.arg("--color").arg("never");
        let output = cmd.output().unwrap();
        if output.status.success() {
            // TODO use regex
            // eprintln!("{}", str::from_utf8(&output.stdout).unwrap());
            CargoTreeOutput::Packages(
                str::from_utf8(&output.stdout)
                    .unwrap()
                    .lines()
                    .filter_map(|s| {
                        if s.contains(" (/") {
                            Some(s.split_whitespace().next().unwrap().to_owned())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            let stderr_first_line = str::from_utf8(&output.stderr)
                .unwrap()
                .lines()
                .next()
                .unwrap();
            let feats = if let Some(feat) = stderr_first_line
                .strip_prefix("error: the package '{pkg}' does not contain this feature: ")
            {
                vec![feat.to_owned()]
            } else if let Some(feats) = stderr_first_line
                .strip_prefix("error: the package '{pkg}' does not contain these features: ")
            {
                feats.split(", ").map(|s| s.to_owned()).collect::<Vec<_>>()
            } else {
                panic!()
            };
            CargoTreeOutput::InvalidFeatures(feats)
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
