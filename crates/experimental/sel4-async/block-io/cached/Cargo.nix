#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-async-block-io-cached";
  dependencies = {
    inherit (versions) lru;
    futures = {
      version = versions.futures;
      default-features = false;
      features = [ "alloc" ];
    };
    inherit (localCrates)
      sel4-async-block-io
    ;
  };
}
