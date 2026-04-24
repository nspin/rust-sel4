//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::{Command, Output};

use cargo_metadata::{Metadata, MetadataCommand, Package, PackageName};
use clap::Parser;
use clap::builder::Str;
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
        assert!(self.cli.exclude.is_empty() || self.cli.include.is_empty());
        let excludes = if !self.cli.exclude.is_empty() {
            self.via_excludes()
        } else if !self.cli.include.is_empty() {
            self.via_includes()
        } else {
            BTreeSet::new()
        };
        let mut ws_args = self.forward_features_args();
        for pkg in excludes.iter() {
            ws_args.push("--exclude".to_owned());
            ws_args.push(pkg.to_string());
        }
        //     let ws = json!({
        // // "rust-analyzer.cargo.extraArgs": [
        // //     "--config", "/home/x/i/rust-sel4/.cargo/config.toml",
        // // 	"--config", "/home/x/i/rust-sel4/.cargo/gen/target/aarch64-sel4-microkit.toml",
        // // 	"--config", "/home/x/i/rust-sel4/.cargo/gen/world/aarch64.microkitDefault.toml",
        // // ],

        //     });
        if self.cli.just_dump_excludes {
            for pkg in excludes.iter() {
                println!("{pkg}");
            }
        } else {
            let ws = self.ws(excludes);
            println!("{ws:#}");
        }
    }

    fn get_orig_config(&self) -> Value {
        let bs = fs::read("/home/x/i/rust-sel4/.vscode/settings.json").unwrap();
        let s = str::from_utf8(&bs).unwrap();
        jsonc_parser::parse_to_serde_value(s, &Default::default()).unwrap()
    }

    fn ws(&self, excludes: BTreeSet<PackageName>) -> Value {
        let mut ws = self.get_orig_config();
        ws.as_object_mut()
            .unwrap()
            .insert("rust-analyzer.cargo.allTargets".to_owned(), json!(false));
        ws.as_object_mut().unwrap().insert(
            "rust-analyzer.server.path".to_owned(),
            json!("/home/x/i/rust-sel4/hacking/vscode/rust-analyzer-defaults-wrapper"),
        );
        ws.as_object_mut().unwrap().insert(
            "rust-analyzer.linkedProjects".to_owned(),
            json!(["/home/x/i/rust-sel4/Cargo.toml"]),
        );
        ws.as_object_mut().unwrap().insert(
            "rust-analyzer.cargo.extraArgs".to_owned(),
            json!(self.forward_config_args()),
        );
        ws.as_object_mut().unwrap().insert(
            "rust-analyzer.cargo.metadataExtraArgs".to_owned(),
            json!(self.forward_config_args()),
        );
        ws.as_object_mut().unwrap().insert(
            "rust-analyzer.cargo.extraEnv".to_owned(),
            json!({
                "__RUST_ANALYZER_WRAPPER__WORKSPACE_ARGS":
                    excludes.iter().map(|x| format!("--exclude {x}")).collect::<Vec<_>>().join(" "),
            }),
        );
        let mut features: Option<Vec<&str>> = None;
        for s in self.cli.features.iter() {
            features.get_or_insert_default().push(s);
        }
        let features_val = if self.cli.all_features {
            if features.is_some() {
                panic!()
            }
            Some(Value::String("all".to_owned()))
        } else if let Some(features) = features {
            Some(Value::Array(
                features
                    .into_iter()
                    .map(|s| Value::String(s.to_owned()))
                    .collect::<Vec<_>>(),
            ))
        } else {
            None
        };
        if let Some(features_val) = features_val {
            ws.as_object_mut()
                .unwrap()
                .insert("rust-analyzer.cargo.features".to_owned(), features_val);
        }
        if self.cli.no_default_features {
            ws.as_object_mut().unwrap().insert(
                "rust-analyzer.cargo.noDefaultFeatures".to_owned(),
                Value::Bool(true),
            );
        }
        json!({
            "folders": [
                { "path": "/home/x/i/rust-sel4" }
            ],
            "settings": ws,
        })
    }

    fn via_includes(&self) -> BTreeSet<PackageName> {
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

        let include_roots = self
            .cli
            .include
            .iter()
            .map(|name| pkg_names[name.as_str()])
            .collect::<BTreeSet<_>>();

        let all_pre = {
            let mut cmd = Command::new("cargo");
            cmd.arg("tree");
            if let Some(s) = self.cli.manifest_path.as_ref() {
                cmd.arg("--manifest-path").arg(s);
            }
            cmd.arg("--prefix").arg("none").arg("--format").arg("{p}");
            cmd.arg("--color").arg("never");
            cmd.arg("--edges=no-build");
            cmd.arg("--edges=no-proc-macro");
            for pkg in self.cli.include.iter() {
                cmd.arg("--package").arg(pkg);
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
                .collect::<BTreeSet<_>>()
        };

        let all = all_pre
            .iter()
            .map(|name| pkg_names[name.as_str()])
            .collect::<BTreeSet<_>>();

        let mut exclude = BTreeSet::new();
        for pkg in workspace_pkgs.iter() {
            let excluded = !all.contains(pkg);
            if excluded {
                exclude.insert((*pkg).clone());
            }
        }
        exclude
    }

    fn via_excludes(&self) -> BTreeSet<PackageName> {
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

        let exclude_roots = self
            .cli
            .exclude
            .iter()
            .map(|name| pkg_names[name.as_str()])
            .collect::<BTreeSet<_>>();

        let fast_exclude_candidates = self
            .get_fast_exclude_candidates()
            .iter()
            .map(|name| pkg_names[name.as_str()])
            .collect::<BTreeSet<_>>();

        let mut exclude = BTreeSet::new();
        for pkg in workspace_pkgs.iter() {
            let excluded = if !fast_exclude_candidates.contains(pkg) {
                false
            } else {
                let raw_deps = self.get_deps(pkg);
                let deps = raw_deps
                    .iter()
                    .map(|name| pkg_names[name.as_str()])
                    .collect::<BTreeSet<_>>();
                deps.intersection(&exclude_roots).count() > 0
            };
            if excluded {
                exclude.insert((*pkg).clone());
            }
        }
        exclude
    }

    fn get_fast_exclude_candidates(&self) -> BTreeSet<String> {
        let output = {
            let mut cmd = Command::new("cargo");
            cmd.arg("tree");
            cmd.args(["--prefix=none", "--format={p}", "--color=never"]);
            if let Some(s) = self.cli.manifest_path.as_ref() {
                cmd.arg("--manifest-path").arg(s);
            }
            cmd.arg("--workspace");
            for pkg in self.cli.exclude.iter() {
                cmd.arg("--invert").arg(pkg);
            }
            cmd
        }
        .output()
        .unwrap();
        assert!(output.status.success());
        CargoTreeOutput::parse_success(&output.stdout)
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
            cmd
        }
        .output()
        .unwrap();
        CargoTreeOutput::parse(&output)
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
