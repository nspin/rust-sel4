#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-async-block-io-disk";
  dependencies = {
    num_enum = { version = versions.num_enum; default-features = false; };
    bytemuck = { version = versions.bytemuck; default-features = false; };
    gpt_disk_types = { version = versions.gpt_disk_types; features = [ "bytemuck" ]; };
    inherit (localCrates)
      sel4-async-block-io
    ;
  };
}
