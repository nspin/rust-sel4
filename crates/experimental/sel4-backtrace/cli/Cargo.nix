#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-backtrace-cli";
  dependencies = {
    inherit (versions) hex;
    addr2line = { version = versions.addr2line; default-features = false; features = [ "loader" ]; };
    clap = { version = versions.clap; features = [ "derive" ]; };
    sel4-backtrace-types = localCrates.sel4-backtrace-types // { features = [ "full" "loader" ]; };
  };
}
