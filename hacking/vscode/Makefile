#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

.PHONY: none
none:

.PHONY: test
test:
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
