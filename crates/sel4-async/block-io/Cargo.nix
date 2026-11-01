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
    embedded-io-async = { version = versions.embedded-io-async; optional = true; };
    thiserror = { version = versions.thiserror; default-features = false; };
    inherit (localCrates) sel4-never-polyfill;
  };
  features = {
    alloc = [];
    default = [ "alloc" ];
    embedded-io-async = [ "dep:embedded-io-async" ];
  };
}
