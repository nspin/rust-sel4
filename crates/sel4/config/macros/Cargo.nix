#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "sel4-config-macros";
  lib.proc-macro = true;
  dependencies = {
    inherit (versions)
      proc-macro2
      quote
    ;
    fallible-iterator = { version = versions.fallible-iterator; default-features = false; };
    syn = { version = versions.syn; features = [ "full" ]; };
    inherit (localCrates)
      sel4-config-types
      sel4-config-data
    ;
  };
}
