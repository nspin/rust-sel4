#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
}:

let
  fetchGit =
    { url, rev, ref ? null
    , submodules ? false
    , local ? null
    , useLocal ? false
    , andThen ? ""
    , extraFilter ? _: _: true
    }:

    assert useLocal -> local != null;

    let
      remote = builtins.fetchGit ({
        inherit url rev submodules;
      } // lib.optionalAttrs (ref != null) {
        inherit ref;
      });
      cleanedSource = lib.cleanSourceWith {
        src = local;
        filter = path: type: lib.cleanSourceFilter path type || extraFilter path type;
      };
      base = if useLocal then cleanedSource else remote;
    in
      base + andThen;

  capdlCommon = rec {
    url = "https://github.com/coliasgroup/capdl.git";
    rev = "26b8d39c17e99e5423763e9ed292a1d123c39f56"; # branch rust
    ref = mkKeepRef rev;
    local = localRoot + "/capdl";
    extraFilter = path: type:
      lib.hasSuffix "/.stack-work" path || lib.hasSuffix "/stack.yaml.lock" path;
  };

  srcRoot = ../../..;

  # TODO
  # localRoot = srcRoot + "/tmp/src";
  localRoot = srcRoot + "/../x";

  mkKeepRef = rev: "refs/tags/keep/${builtins.substring 0 32 rev}";

in rec {
  inherit srcRoot localRoot;
  inherit fetchGit mkKeepRef;

  # NOTE: Be sure to keep the commit hashes in the top-level README up-to-date.

  seL4 = fetchGit {
    url = "https://github.com/seL4/seL4.git";
    rev = "62e345c240a15671484530b4e6e069ce266c28d7"; # 17.0.0
    local = localRoot + "/seL4";
  };

  microkit = fetchGit {
    url = "https://github.com/seL4/microkit.git";
    rev = "5e46573c89c28f353634e5ec198458ecf4bab286"; # before 2.4.0
    local = localRoot + "/microkit";
    extraFilter = path: type:
      lib.hasSuffix "/target" path;
  };

  acacia = fetchGit rec {
    url = "https://github.com/nspin/microkit_acacia";
    rev = "f11d2fae4e2ea4037024e256256d721ece65ee0e";
    ref = mkKeepRef rev;
    # XXX REPRO
    # See the above lines for the commit I'm using.
    # To use local checkout of microkit_acacia, uncomment the following two lines.
    # local = ../../../../path/to/local/checkout/of/microkit_acacia;
    # useLocal = true;
  };

  # micropython submodule is too big for this to be worth it
  # sddf = fetchGit (lionsosAttrs // {
  #   submodules = true;
  #   andThen = "/dep/sddf";
  #   local = localRoot + "/lionsos/dep/sddf";
  # });

  sddf = fetchGit {
    url = "https://github.com/au-ts/sddf";
    rev = "51c964da31b67d709c1c4511322ae053b2344225"; # main
    # ref = null;
    local = localRoot + "/lionsos/dep/sddf";
  };

  lionsosAttrs = {
    url = "https://github.com/au-ts/lionsos";
    rev = "89051d0387cc970cef70857d1816e9134330120b"; # main
    # ref = null;
    local = localRoot + "/lionsos";
  };

  lionsos = fetchGit lionsosAttrs;

  capdlTool = fetchGit (capdlCommon // {
    andThen = "/capDL-tool";
  });

  pythonCapDLTool = fetchGit (capdlCommon // {
    andThen = "/python-capdl-tool";
  });

  objectSizes = fetchGit (capdlCommon // {
    andThen = "/object_sizes";
  });
}
