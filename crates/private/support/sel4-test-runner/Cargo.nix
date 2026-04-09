#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions }:

mk {
  package.name = "sel4-test-runner";
  dependencies = {
    inherit (versions)
      anyhow
      tempfile
    ;
    clap = { version = versions.clap; features = [ "derive" ]; };
  };
}
