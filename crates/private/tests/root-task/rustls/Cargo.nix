#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, ringWith, rustlsWith }:

mk {
  package.name = "tests-root-task-rustls";
  dependencies = {
    rustls = (localCrates.rustls or {}) // rustlsWith [];
    ring = (localCrates.ring or {}) // ringWith [];
    getrandom = {
      version = versions.getrandom;
      features = [ "custom" ];
    };
    rand = {
      version = versions.rand;
      default-features = false;
      features = [
        "small_rng"
      ];
    };
    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "nosys"
        "all-symbols"
        "sel4-panicking-env"
      ];
    };
    inherit (localCrates)
      sel4
      sel4-root-task
      sel4-logging
      sel4-async-network-rustls
    ;
  };
}
