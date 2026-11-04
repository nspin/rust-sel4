#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sddf-async-block-io";
  dependencies = {
    inherit (versions) heapless;
    async-unsync = { version = versions.async-unsync; default-features = false; };
    thiserror = { version = versions.thiserror; default-features = false; };

    inherit (localCrates)
      sddf
      sel4-async-block-io
      sel4-shared-memory
    ;
  };
}
