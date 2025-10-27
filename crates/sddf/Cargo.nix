#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sddf";
  dependencies = {
    inherit (localCrates)
      sddf-sys
      sel4-config
      sel4-immutable-cell
    ;
    sel4-microkit-base = localCrates.sel4-microkit-base // {
      optional = true;
    };
  };
  features = {
    "sel4-microkit-base" = [ "dep:sel4-microkit-base" ];
  };
}
