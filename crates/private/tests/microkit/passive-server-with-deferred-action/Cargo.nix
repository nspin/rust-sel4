#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-microkit-passive-server-with-deferred-action";
  dependencies = {
    inherit (localCrates)
      sel4-test-microkit
    ;
    sel4-microkit = localCrates.sel4-microkit // { features = [ "alloc" ]; };
  };
}
