#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

workspaces_dir := workspaces

private_workspaces_dir := $(workspaces_dir)/private

ws := .code_workspace

.PHONY: none
none:

.PHONY: default-excludes
default-excludes:
	cargo run -p cargo-env-wrapper -- \
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
	cargo run -p cargo-env-wrapper -- \
		-i microkit-http-server-example-server \
		--config .cargo/gen/target/aarch64-sel4-microkit.toml \
		--config .cargo/gen/world/aarch64.microkitDefault.toml \
		-o $(private_workspaces_dir)/ukit$(ws)
