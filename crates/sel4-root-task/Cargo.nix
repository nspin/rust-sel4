#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-root-task";
  dependencies = {
    inherit (localCrates)
      sel4
      sel4-immediate-sync-once-cell
      sel4-panicking-env
      sel4-dlmalloc
      sel4-sync
      sel4-ctors-dtors
      sel4-root-task-macros
    ;
    sel4-panicking = localCrates.sel4-panicking // { optional = true; };
    sel4-runtime-common = localCrates.sel4-runtime-common // { features = [ "tls" "start" ]; };
  };
  features = {
    default = [
      "no-std"
      "unwinding"
    ];
    full = [
      "default"
      "alloc"
    ];
    no-std = [
      "dep:sel4-panicking"
    ];
    std = [
      "unwinding"
    ];
    unwinding = [
      "sel4-panicking?/unwinding"
      "sel4-runtime-common/unwinding"
    ];
    alloc = [
      "sel4-panicking?/alloc"
    ];
    single-threaded = [
      "sel4/single-threaded"
    ];
  };
}
