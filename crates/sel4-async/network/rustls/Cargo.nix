#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, ringWith, rustlsWith }:

mk {
  package.name = "sel4-async-network-rustls";
  dependencies = {
    inherit (localCrates)
      sel4-async-network
      sel4-async-network-mbedtls # TODO
    ;
    inherit (versions) log;
    rustls = rustlsWith [] // (localCrates.rustls or {});
    ring = ringWith [] // (localCrates.ring or {}); # just to force "less-safe-getrandom-custom-or-rdrand" feature
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
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
  };
}
