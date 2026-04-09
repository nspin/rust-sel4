#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-test-runner";
  dependencies = {
    inherit (versions)
      anyhow
      tempfile
      serde_json
      object
    ;
    clap = { version = versions.clap; features = [ "derive" ]; };
    sel4-config-types = localCrates.sel4-config-types // {
      features = [ "serde" ];
    };
  };
}
