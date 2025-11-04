{ lib
, stdenv
, writers
, linkFarm

, crateUtils
, pruneLockfile

, defaultRustEnvironment

, rustEnvironment ? defaultRustEnvironment

, crates
, globalPatchSection
}:

let
  topLevelPath = ../../..;

  blockList = [
    "sel4-config-data"
    "sel4-panicking"
    "sel4-reset" # not yet implement for x86
    "sel4-newlib" # requires env
    "sel4-kernel-loader" # requires env
    "tests-root-task-dafny-core"
  ];

  members = lib.filterAttrs
    (_name: crate: !(lib.any (bad: lib.hasAttr bad crate.nonOptionalClosure) blockList))
    crates;

  exclude = lib.filterAttrs
    (name: _crate: !(lib.hasAttr name members))
    crates;

  relPath = path: lib.replaceStrings [ "${toString topLevelPath}/" ] [ "" ] (toString path);

  manifest = writers.writeTOML "Cargo.toml" {
    workspace = {
      resolver = "3";
      members = lib.naturalSort (lib.mapAttrsToList (_name: crate: relPath crate.path) members);
      exclude = lib.naturalSort (lib.mapAttrsToList (_name: crate: relPath crate.path) exclude);
    };
    patch = globalPatchSection;
  };

  lockfile = pruneLockfile {
    inherit (rustEnvironment) rustToolchain vendoredSuperLockfile;
    rootCrates = lib.attrValues members;
    extraManifest = {
      patch = globalPatchSection;
    };
  };

  toolchainFile = writers.writeTOML "rust-toolchain.toml" {
    toolchain = {
      inherit (rustEnvironment) channel;
      components = [ "rust-src" "rustc-dev" "llvm-tools-preview" "rust-analyzer" ];
      profile = "default";
    };
  };

  configFile = writers.writeTOML "rust-toolchain.toml" {
    unstable = {
      unstable-options = true;
    };
    # env = {
    #   LIBCLANG_PATH = libclangPath;
    #   HOST_CC = "${stdenv.cc.targetPrefix}gcc";
    # };
  };

  settingsFile = writers.writeJSON "settings.json" {
    "rust-analyzer.server.path" = "rust-analyzer";
    "terminal.integrated.cwd" = toString topLevelPath;
    "rust-analyzer.cargo.targetDir" = true; # use subdir of outer target-dir
  };

  workspaceName = "default-workspace";

  mkSubdir = prefix: map (v: v // { name = "${prefix}/${v.name}"; });

  dir = linkFarm "workspace" (mkSubdir workspaceName [
    { name = ".vscode/settings.json"; path = settingsFile; }
    { name = ".cargo/config.toml"; path = configFile; }
    { name = "rust-toolchain.toml"; path = toolchainFile; }
    { name = "rustfmt.toml"; path = toString (topLevelPath + "/rustfmt.toml"); }
    { name = "Cargo.toml"; path = manifest; }
    { name = "Cargo.lock"; path = lockfile; }
    { name = "crates"; path = toString (topLevelPath + "/crates"); }
    { name = "target"; path = toString (topLevelPath + "/tmp/vscode-env/default"); }
  ]);

in
rec {

  inherit dir;
  inherit workspaceName;
  inherit members;

}
