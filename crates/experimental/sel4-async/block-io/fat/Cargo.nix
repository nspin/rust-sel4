#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, fatSource }:

mk {
  package.name = "sel4-async-block-io-fat";
  dependencies = {
    inherit (versions) log heapless lru aligned;
    hex = { version = versions.hex; default-features = false; };
    embedded-fatfs = fatSource // {
      default-features = false;
      features = [ "lfn" "log" ];
    };
    block-device-driver = fatSource;
    block-device-adapters = fatSource;
    inherit (localCrates)
      sel4-async-block-io
    ;
  };
}
