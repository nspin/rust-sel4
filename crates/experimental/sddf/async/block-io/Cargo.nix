#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sddf-async-block-io";
  dependencies = {
    inherit (versions) log;

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
      ];
    };

    async-unsync = { version = versions.async-unsync; default-features = false; };

    inherit (localCrates)
      sel4-shared-ring-buffer
      sel4-shared-ring-buffer-block-io-types
      sel4-abstract-allocator
      sel4-async-block-io
      sel4-shared-memory
    ;

    sel4-bookkeeping = localCrates.sel4-bookkeeping // { features = [ "async-unsync" ]; };
  };
}
