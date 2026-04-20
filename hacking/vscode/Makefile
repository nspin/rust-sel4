#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

.PHONY: none
none:

.PHONY: test1
test1:
	cargo run -p cargo-env-wrapper -- \
		--manifest-path=../../Cargo.toml \
		foo \
		-e sel4-config-data \
		-e sel4-test-harness \
		-e tests-root-task-dafny-core \
		-e lionsos-sys \
		-e sddf-sys \
		-e sel4-newlib \
	> $(default_excludes)

.PHONY: test2
test2:
	cargo run -p cargo-env-wrapper -- \
		--manifest-path=../../Cargo.toml \
		foo \
		-i microkit-http-server-example-server \
	> $(default_excludes)
