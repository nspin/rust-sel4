#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sddf-async-io";
  dependencies = {
    thiserror = { version = versions.thiserror; default-features = false; };
    inherit (localCrates)
      sddf
      sel4-async-io
    ;
  };
}
