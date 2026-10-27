#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-sys";
  package.links = "sel4";
  package.build = "build/main.rs";
  dependencies = {
    inherit (versions) log;
    inherit (localCrates)
      sel4-config
      sel4-bitfield-ops
    ;
  };
  build-dependencies = {
    inherit (versions)
      proc-macro2 quote
      bindgen xmltree glob
      regex pest pest_derive
    ;
    prettyplease = { version = versions.prettyplease; optional = true; };
    syn = { version = versions.syn; features = [ "parsing" ]; optional = true; };
    inherit (localCrates)
      sel4-build-env
      sel4-config
      sel4-config-data
    ;
  };
  features = {
    prettyplease = [ "dep:prettyplease" "dep:syn" ];
  };
}
