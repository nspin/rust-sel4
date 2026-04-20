#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

default_excludes := default-excludes.txt

.PHONY: none
none:

.PHONY: test
test: $(default_excludes)

$(default_excludes):
	cargo run -p cargo-env-wrapper -- --manifest-path=../../Cargo.toml
