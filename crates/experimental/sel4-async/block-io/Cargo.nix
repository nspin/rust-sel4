#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-async-block-io";
  dependencies = {
    futures = {
      version = versions.futures;
      default-features = false;
    };
    inherit (localCrates) sel4-never-polyfill;
  };
  features = {
    alloc = [];
    default = [ "alloc" ];
  };
}
