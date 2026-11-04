#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, serdeWith, smoltcpWith }:

mk {
  package.name = "microkit-http-server-example-server";

  dependencies = {
    inherit (versions) log rtcc one-shot-mutex;

    futures = {
      version = versions.futures;
      default-features = false;
      features = [
        "async-await"
        "alloc"
      ];
    };

    smoltcp = smoltcpWith [
      "log"
    ];

    async-unsync = { version = versions.async-unsync; default-features = false; };

    inherit (localCrates)
      sel4-logging
      sel4-immediate-sync-once-cell
      sel4-microkit-driver-adapters
      sel4-never-polyfill
      sel4-driver-interfaces
      sel4-async-single-threaded-executor
      sel4-async-network
      sel4-async-time
      sddf
      sddf-smoltcp
      sddf-async-block-io
      sel4-async-block-io
      sel4-async-block-io-cached
      sel4-async-block-io-disk
    ;

    sel4-newlib = localCrates.sel4-newlib // {
      features = [
        "detect-libc"
        "link-libc"
        "link-libnosys"
        "all-symbols"
      ];
    };

    sel4-microkit = localCrates.sel4-microkit // { features = [ "alloc" ]; };

    # The sDDF timer client library uses libsel4's IPC buffer pointer.
    sel4 = localCrates.sel4 // { features = [ "exposed-state" "non-thread-local-state" ]; };

    microkit-http-server-example-server-core = localCrates.microkit-http-server-example-server-core // {
      features = [
        # "debug"
      ];
    };
  };

  build-dependencies = {
    inherit (versions) rcgen;
  };
}
