#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, rustlsWith }:

mk {
  package.name = "sel4-async-network-rustls";
  dependencies = {
    inherit (localCrates)
      sel4-async-network
      sel4-async-network-mbedtls # TODO
    ;
    inherit (versions) log;
    rustls = (localCrates.rustls or {}) // rustlsWith [];
    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "alloc"
      ];
    };
    rand = {
      version = "0.8.5";
      default-features = false;
      features = [
        "small_rng"
      ];
    };
  };
}
