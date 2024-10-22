#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-musl";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-musl
      sel4-linux-syscall-types
    ;
  };
}
