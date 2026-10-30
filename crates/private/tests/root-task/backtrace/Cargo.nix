#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "tests-root-task-backtrace";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-backtrace-embedded-debug-info
      sel4-backtrace-symbolize
      sel4-test-root-task
    ;
    sel4-root-task = localCrates.sel4-root-task // { features = [ "alloc" ]; };
    sel4-backtrace-collect = localCrates.sel4-backtrace-collect // { features = [ "unwinding" ]; };
  };
}
