#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-p";
  dependencies = {
    inherit (localCrates)
      sel4
      # sel4-root-task
    ;
    sel4-root-task = localCrates.sel4-root-task // { features = [ "alloc" ]; };
  };
}
