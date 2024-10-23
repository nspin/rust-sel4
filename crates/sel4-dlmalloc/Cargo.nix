#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-dlmalloc";
  dependencies = {
    inherit (versions)
      dlmalloc
      lock_api
    ;
    inherit (localCrates)
      sel4-static-heap
    ;
  };
}
