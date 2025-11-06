#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-bare-metal-test-runner";
  dependencies = {
    inherit (versions)
      one-shot-mutex
    ;
    syscalls = {
      version = versions.syscalls;
      default-features = false;
      features = [ "all" ];
    };
    inherit (localCrates)
      sel4-immediate-sync-once-cell
      sel4-panicking-env
      sel4-dlmalloc
    ;
    sel4-panicking = localCrates.sel4-panicking // { features = [ "alloc" "personality" "panic-handler" ]; };
    sel4-runtime-common = localCrates.sel4-runtime-common // { features = [ "full" ]; };
  };
}
