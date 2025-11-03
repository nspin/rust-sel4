#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, writeText
, crateUtils
, sources
}:

let
  workspaceDir = sources.srcRoot;

  workspaceManifest = builtins.fromTOML (builtins.readFile workspaceManifestPath);
  workspaceManifestPath = workspaceDir + "/Cargo.toml";
  workspaceMemberPaths = map (member: workspaceDir + "/${member}") workspaceManifest.workspace.members;

  globalPatchSection = workspaceManifest.patch;

  overridesForMkCrate = {
    sel4-sys = {
      extraPaths = [
        "build"
      ];
    };
    sel4-bitfield-parser = {
      extraPaths = [
        "grammar.pest"
      ];
    };
    sel4-kernel-loader = {
      extraPaths = [
        "asm"
      ];
    };
    tests-root-task-c = {
      extraPaths = [
        "cbits"
      ];
    };
    banscii-assistant-core = {
      resolveLinks = true;
      extraPaths = [
        "assets"
      ];
    };
    ring = {
      resolveLinks = true;
      extraPaths = [
        "tests"
        "include"
        "crypto"
        "third_party"
        ".git/HEAD"
      ];
    };
    offset-allocator = {
      resolveLinks = true;
      extraPaths = [
        "README.md"
      ];
    };
  };

  unAugmentedCrates = lib.listToAttrs (lib.forEach workspaceMemberPaths (cratePath: rec {
    name = (crateUtils.crateManifest cratePath).package.name;
    value = crateUtils.mkCrate cratePath (overridesForMkCrate.${name} or {});
  }));

  augmentCrates = crates: lib.fix (selfCrates:
    lib.flip lib.mapAttrs crates (_: crate:
      let
        apply = f:
          lib.mapAttrsToList
            (k: v: lib.getAttr k selfCrates)
            (f crate);
      in
        lib.fix (selfCrate: crate // {
          closure = {
            "${crate.name}" = selfCrate;
          } // lib.foldl'
            (acc: crate': acc // crate'.closure)
            {}
            (apply crateUtils.getDeps.all);
          nonOptionalClosure = {
            "${crate.name}" = selfCrate;
          } // lib.foldl'
            (acc: crate': acc // crate'.nonOptionalClosure)
            {}
            (apply crateUtils.getDeps.nonOptional);
          normalNonOptionalClosure = {
            "${crate.name}" = selfCrate;
          } // lib.foldl'
            (acc: crate': acc // crate'.normalNonOptionalClosure)
            {}
            (lib.filter
              (crate'': (!(crate''.manifest.lib.proc-macro or false)))
              (apply crateUtils.getDeps.normalNonOptional));
          normalNonOptionalClosureWithProcMacros = {
            "${crate.name}" = selfCrate;
          } // lib.foldl'
            (acc: crate':
              acc //
              { "${crate'.name }" = crate'; } //
              lib.optionalAttrs
                (!(crate'.manifest.lib.proc-macro or false))
                crate'.normalNonOptionalClosureWithProcMacros)
            {}
            (apply crateUtils.getDeps.normalNonOptional);
        })
    )
  );

  crates = augmentCrates unAugmentedCrates;

  isPrivate = crateName: builtins.match "^sel4-.*" crateName == null;

  publicCrates = lib.flip lib.filterAttrs crates (k: v: !isPrivate k);

  publicCratesTxt = writeText "public-crates.txt"
    (lib.concatMapStrings (crateName: "${crateName}\n") (lib.attrNames publicCrates));

in {
  inherit crates overridesForMkCrate globalPatchSection publicCrates publicCratesTxt;
}
