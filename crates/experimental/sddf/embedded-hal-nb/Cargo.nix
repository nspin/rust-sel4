#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sddf-embedded-hal-nb";
  dependencies = {
    inherit (versions) embedded-hal-nb;
    thiserror = { version = versions.thiserror; default-features = false; };
    inherit (localCrates)
      sddf
    ;
  };
}
