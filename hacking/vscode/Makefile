#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

workspaces_dir := workspaces

private_workspaces_dir := $(workspaces_dir)/private

ws := .code-workspace

.PHONY: none
none:

.PHONY: default-excludes
default-excludes:
	cargo run -p create-vscode-workspace -- \
		-e sel4-config-data \
		-e sel4-test-harness \
		-e tests-root-task-dafny-core \
		-e lionsos-sys \
		-e sddf-sys \
		-e sel4-newlib \
		--just-dump-excludes \
		-o $(default_excludes)

.PHONY: priv
priv:
	cargo run -p create-vscode-workspace -- \
		-i microkit-http-server-example-server \
		--config .cargo/gen/target/aarch64-sel4-microkit.toml \
		--config .cargo/gen/world/aarch64.microkitDefault.toml \
		-o $(private_workspaces_dir)/ukit$(ws)

.PHONY: t
t:
	cargo run -p create-vscode-workspace -- \
		--include-dependents sel4 \
		-e tests-root-task-dafny-core \
		--just-dump-excludes \
		-o $(private_workspaces_dir)/foo.txt
