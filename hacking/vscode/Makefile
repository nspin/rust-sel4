#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

.PHONY: none
none:

.PHONY: t1
t1:
	cargo run -p cargo-env-wrapper -- \
		--manifest-path=../../Cargo.toml \
		foo \
		-e sel4-config-data \
		-e sel4-test-harness \
		-e tests-root-task-dafny-core \
		-e lionsos-sys \
		-e sddf-sys \
		-e sel4-newlib \
		-d \
	> $(default_excludes)

.PHONY: t2
t2:
	cargo run -p cargo-env-wrapper -- \
		--manifest-path=../../Cargo.toml \
		foo \
		-i microkit-http-server-example-server \
	> $(default_excludes)

gen := $(abspath ../../.cargo/gen)
cargo_config := $(abspath ../../.cargo)

.PHONY: t3
t3:
	cargo run -p cargo-env-wrapper -- \
		--manifest-path=../../Cargo.toml \
		-i microkit-http-server-example-server \
		--config $(gen)/target/aarch64-sel4-microkit.toml \
		--config $(gen)/world/aarch64.microkitDefault.toml

.PHONY: t4
t4:
	cargo build -p cargo-env-wrapper && \
	../target/debug/cargo-env-wrapper \
		--manifest-path=../../Cargo.toml \
		-i microkit-http-server-example-server \
		--config $(cargo_config)/config.toml \
		--config $(gen)/target/aarch64-sel4-microkit.toml \
		--config $(gen)/world/aarch64.microkitDefault.toml

.PHONY: t5
t5:
	cargo build -p cargo-env-wrapper && \
	../target/debug/cargo-env-wrapper \
		--manifest-path=../../Cargo.toml \
		-i microkit-http-server-example-server \
		--config $(cargo_config)/config.toml \
		--config $(gen)/target/aarch64-sel4-microkit.toml \
		--config $(gen)/world/aarch64.microkitDefault.toml
