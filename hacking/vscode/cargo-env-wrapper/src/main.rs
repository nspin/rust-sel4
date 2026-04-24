//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use cargo_metadata::{Metadata, MetadataCommand, PackageName};
use clap::Parser;
use serde_json::{Value, json};

fn main() {
    Env::get().run()
}

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

    #[arg(long, short = 'i')]
    include: Vec<String>,

    #[arg(long, short = 'e')]
    exclude: Vec<String>,

    #[arg(long, short = 'd')]
    just_dump_excludes: bool,
}

struct Env {
    cli: Cli,
}

enum CargoTreeOutput {
    Packages(BTreeSet<String>),
    InvalidFeatures(Vec<String>),
}

impl CargoTreeOutput {
    fn parse(output: &Output) -> Self {
        if output.status.success() {
            CargoTreeOutput::Packages(Self::parse_success(&output.stdout))
        } else {
            CargoTreeOutput::InvalidFeatures(Self::parse_failure(&output.stderr))
        }
    }

    fn assume_success(output: &Output) -> BTreeSet<String> {
        assert!(output.status.success());
        Self::parse_success(&output.stdout)
    }

    fn parse_success(stdout: &[u8]) -> BTreeSet<String> {
        // TODO use regex
        str::from_utf8(stdout)
            .unwrap()
            .lines()
            .filter_map(|s| {
                if s.contains(" (/") {
                    Some(s.split_whitespace().next().unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
    }

    fn parse_failure(stderr: &[u8]) -> Vec<String> {
        let stderr_first_line = str::from_utf8(stderr).unwrap().lines().next().unwrap();
        if let Some(s) = stderr_first_line
            .strip_prefix("error: the package '{pkg}' does not contain this feature: ")
        {
            vec![s.to_owned()]
        } else if let Some(s) = stderr_first_line
            .strip_prefix("error: the package '{pkg}' does not contain these features: ")
        {
            s.split(", ").map(|s| s.to_owned()).collect::<Vec<_>>()
        } else {
            panic!()
        }
    }
}

struct WorkspacePackages {
    pkgs: BTreeSet<PackageName>,
    pkgs_by_name: BTreeMap<String, PackageName>,
}

impl WorkspacePackages {
    fn from_metadata(metadata: &Metadata) -> Self {
        let pkgs = metadata
            .workspace_packages()
            .iter()
            .map(|pkg| pkg.name.clone())
            .collect::<BTreeSet<_>>();

        let mut pkgs_by_name = BTreeMap::new();
        for pkg in pkgs.iter() {
            pkgs_by_name.insert(pkg.to_string(), pkg.clone());
        }

        WorkspacePackages { pkgs, pkgs_by_name }
    }

    fn iter(&self) -> impl Iterator<Item = &PackageName> {
        self.pkgs.iter()
    }

    fn by_name(&self, name: impl AsRef<str>) -> &PackageName {
        &self.pkgs_by_name[name.as_ref()]
    }
}

impl Env {
    fn get() -> Self {
        let cli = Cli::parse();
        Self { cli }
    }

    fn run(&self) {
        let workspace_packages = self.workspace_packages();

        assert!(self.cli.exclude.is_empty() || self.cli.include.is_empty());
        let excludes = if !self.cli.exclude.is_empty() {
            self.via_excludes(&workspace_packages)
        } else if !self.cli.include.is_empty() {
            self.via_includes(&workspace_packages)
        } else {
            BTreeSet::new()
        };

        if self.cli.just_dump_excludes {
            for pkg in excludes.iter() {
                println!("{pkg}");
            }
        } else {
            let ws = self.ws(excludes);
            println!("{ws:#}");
        }
    }

    fn get_orig_settings(&self) -> Value {
        let bs = fs::read("/home/x/i/rust-sel4/.vscode/settings.json").unwrap();
        let s = str::from_utf8(&bs).unwrap();
        jsonc_parser::parse_to_serde_value(s, &Default::default()).unwrap()
    }

    fn ws(&self, excludes: BTreeSet<&PackageName>) -> Value {
        let exclude_args = excludes
            .iter()
            .map(|x| format!("--exclude={x}"))
            .collect::<Vec<_>>()
            .join(" ");

        let mut new_settings = json!({
            "rust-analyzer.cargo.allTargets": false,
            "rust-analyzer.server.path": "/home/x/i/rust-sel4/hacking/vscode/rust-analyzer-defaults-wrapper",
            "rust-analyzer.linkedProjects": [
                "/home/x/i/rust-sel4/Cargo.toml",
            ],
            "rust-analyzer.cargo.extraArgs": self.forward_config_args(),
            "rust-analyzer.cargo.metadataExtraArgs": self.forward_config_args(),
            "rust-analyzer.cargo.extraEnv": {
                "__RUST_ANALYZER_WRAPPER__WORKSPACE_ARGS": exclude_args,
            },
        });

        let new_settings_obj = new_settings.as_object_mut().unwrap();

        let features_val = if self.cli.all_features {
            if !self.cli.features.is_empty() {
                panic!()
            }
            Some(json!("all"))
        } else if !self.cli.features.is_empty() {
            Some(json!(self.cli.features))
        } else {
            None
        };
        if let Some(features_val) = features_val {
            new_settings_obj.insert("rust-analyzer.cargo.features".to_owned(), features_val);
        }
        if self.cli.no_default_features {
            new_settings_obj.insert(
                "rust-analyzer.cargo.noDefaultFeatures".to_owned(),
                json!(true),
            );
        }

        let mut settings = self.get_orig_settings();
        settings.as_object_mut().unwrap().append(new_settings_obj);

        json!({
            "folders": [
                { "path": "/home/x/i/rust-sel4" }
            ],
            "settings": settings,
        })
    }

    fn via_includes<'a>(
        &self,
        workspace_packages: &'a WorkspacePackages,
    ) -> BTreeSet<&'a PackageName> {
        let transitive_includes = {
            let output = {
                let mut cmd = self.cargo_tree_base_cmd();
                cmd.arg("--edges=no-build");
                cmd.arg("--edges=no-proc-macro");
                for pkg in self.cli.include.iter() {
                    cmd.arg("--package")
                        .arg(workspace_packages.by_name(pkg).as_str());
                }
                cmd
            }
            .output()
            .unwrap();
            CargoTreeOutput::assume_success(&output)
        }
        .iter()
        .map(|name| workspace_packages.by_name(name))
        .collect::<BTreeSet<_>>();

        workspace_packages
            .iter()
            .filter(|pkg| !transitive_includes.contains(pkg))
            .collect::<BTreeSet<_>>()
    }

    fn via_excludes<'a>(
        &self,
        workspace_packages: &'a WorkspacePackages,
    ) -> BTreeSet<&'a PackageName> {
        let exclude_roots = self
            .cli
            .exclude
            .iter()
            .map(|name| workspace_packages.by_name(name))
            .collect::<BTreeSet<_>>();

        let fast_exclude_candidates = self
            .get_fast_exclude_candidates()
            .iter()
            .map(|name| workspace_packages.by_name(name))
            .collect::<BTreeSet<_>>();

        let mut exclude = BTreeSet::new();
        for pkg in workspace_packages.iter() {
            let excluded = if !fast_exclude_candidates.contains(pkg) {
                false
            } else {
                let raw_deps = self.get_deps(pkg);
                let deps = raw_deps
                    .iter()
                    .map(|name| workspace_packages.by_name(name))
                    .collect::<BTreeSet<_>>();
                deps.intersection(&exclude_roots).count() > 0
            };
            if excluded {
                exclude.insert(pkg);
            }
        }
        exclude
    }

    fn get_fast_exclude_candidates(&self) -> BTreeSet<String> {
        let output = {
            let mut cmd = self.cargo_tree_base_cmd();
            cmd.arg("--workspace");
            for pkg in self.cli.exclude.iter() {
                cmd.arg("--invert").arg(pkg);
            }
            cmd
        }
        .output()
        .unwrap();
        CargoTreeOutput::assume_success(&output)
    }

    fn get_deps(&self, pkg: &PackageName) -> BTreeSet<String> {
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
        let output = {
            let mut cmd = self.cargo_tree_base_cmd();
            cmd.arg("--package").arg(pkg.as_ref());
            cmd.args(self.forward_args_with_feature_filter(|feat| {
                !exclude_features
                    .iter()
                    .any(|excluded_feat| feat == excluded_feat.as_ref())
            }));
            cmd
        }
        .output()
        .unwrap();
        CargoTreeOutput::parse(&output)
    }

    fn cargo_tree_base_cmd(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.args(["tree", "--prefix=none", "--format={p}", "--color=never"]);
        if let Some(s) = self.cli.manifest_path.as_ref() {
            cmd.arg("--manifest-path").arg(s);
        }
        cmd
    }

    fn workspace_packages(&self) -> WorkspacePackages {
        let metadata = {
            let mut cmd = MetadataCommand::new();
            if let Some(s) = self.cli.manifest_path.as_ref() {
                cmd.manifest_path(s);
            }
            cmd.other_options(self.forward_features_args());
            cmd.no_deps();
            cmd.exec().unwrap()
        };
        WorkspacePackages::from_metadata(&metadata)
    }

    fn forward_args_with_feature_filter(
        &self,
        feature_filter: impl Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut args = self.forward_features_args_with_feature_filter(&feature_filter);
        args.extend(self.forward_config_args());
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

    fn forward_config_args(&self) -> Vec<String> {
        let mut args = vec![];
        if let Some(s) = self.cli.target.as_ref() {
            args.push("--config".to_owned());
            args.push(format!("build.target=\"{s}\""));
        }
        for s in self.cli.config.iter() {
            args.push("--config".to_owned());
            args.push(s.to_owned());
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
