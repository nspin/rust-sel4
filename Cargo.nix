#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, localCrates, defaultFrontmatter, nonOptionalClosures }:

let
  defaultMembers =
    let
      blockList = [
        "sel4-config-data"
        "sel4-test-harness"
        "sel4-reset" # not yet implement for x86
        "sel4-newlib" # requires newlib
        "tests-root-task-dafny-core" # requires input via env
      ];
    in
      lib.filterAttrs
        (k: _: !(lib.any (blocked: lib.hasAttr blocked nonOptionalClosures.${k})) blockList)
        nonOptionalClosures;

in {
  nix.frontmatter = defaultFrontmatter;
  nix.formatPolicyOverrides = [
    {
      table_rules = [
        {
          path_regex = ""; # top-level table
          key_ordering.back = [ "patch" ];
        }
      ];
    }
  ];
  workspace = {
    resolver = "3";
    default-members = lib.naturalSort (lib.mapAttrsToList (k: _: localCrates.${k}.path) defaultMembers);
    members = lib.naturalSort (lib.mapAttrsToList (_: v: v.path) localCrates);
  };
  patch.crates-io = {
    ring = localCrates.ring or  {
      git = "https://github.com/coliasgroup/ring.git";
      rev = "0f749acc5d5a8310dfc3ff985df04056f497fc1b"; # branch sel4
    };
  };
}
