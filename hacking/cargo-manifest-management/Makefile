#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

blueprint := blueprint.json

tools_manifest_path_arg := --manifest-path=tool/Cargo.toml

run := cargo run \
	$(tools_manifest_path_arg) -p manage-cargo-manifests --

run_allowlist := cargo run \
	$(tools_manifest_path_arg) -p manage-direct-dependency-allow-list --

run_vscode_exclude := cargo run \
	$(tools_manifest_path_arg) -p manage-vscode-exclude-list --

project_manifest_path_arg := --manifest-path=../../Cargo.toml

allowlist := direct-dependency-allow-list.toml

run_allowlist_check_workspace := $(run_allowlist) check-workspace --allowlist $(allowlist) $(project_manifest_path_arg)
run_allowlist_update := $(run_allowlist) update --allowlist $(allowlist) -o $(allowlist)

vscode_excludes_path := ../vscode/excludes.txt
vscode_args := \
	$(project_manifest_path_arg) \
	-o $(vscode_excludes_path)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf tool/target $(blueprint)

.PHONY: $(blueprint)
$(blueprint):
	nix-build -A blueprintJSON --out-link $@

.PHONY: update
update: $(blueprint)
	$(run) --blueprint $<
	$(run_vscode_exclude) $(vscode_args)

.PHONY: check
check: $(blueprint)
	$(run) --blueprint $< --just-check
	$(run_vscode_exclude) $(vscode_args) --just-check

.PHONY: check-workspace-direct-dependencies
check-workspace-direct-dependencies:
	$(run_allowlist_check_workspace)

.PHONY: update-direct-dependency-allowlist
update-direct-dependency-allowlist:
	$(run_allowlist_update)

.PHONY: check-direct-dependency-allow-list
check-direct-dependency-allow-list:
	$(run_allowlist_update) --check
