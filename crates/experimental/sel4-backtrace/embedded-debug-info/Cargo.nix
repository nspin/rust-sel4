#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, objectWith }:

mk {
  package.name = "sel4-backtrace-embedded-debug-info";
  dependencies = {
    addr2line = { version = versions.addr2line; default-features = false; };
    gimli = { version = versions.gimli; default-features = false; features = [ "read" ]; };
    object = objectWith [];
    thiserror = { version = versions.thiserror; default-features = false; };
    inherit (localCrates)
      sel4-phdrs
      sel4-phdrs-patched
    ;
  };
}
