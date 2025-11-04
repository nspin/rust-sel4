{ lib
, stdenv
, hostPlatform
, writers
, linkFarm

, defaultRustEnvironment
, defaultRustTargetTriple
, rustEnvironment ? defaultRustEnvironment

, globalPatchSection
, pruneLockfile
, crateUtils
, crates

, libclangPath

, shell
}:

let
  targetTriple = defaultRustTargetTriple;

  topLevelPath = ../../../../..;

  targetDirBase = topLevelPath + "/tmp/vscode-env/${hostPlatform.config}";

  relPath = path: lib.replaceStrings [ "${toString topLevelPath}/" ] [ "" ] (toString path);

  toolchainFile = writers.writeTOML "rust-toolchain.toml" {
    toolchain = {
      inherit (rustEnvironment) channel;
      components = [ "rust-src" "rustc-dev" "llvm-tools-preview" "rust-analyzer" ];
      profile = "default";
    };
  };

  blockList = [
    "sel4-config-data"
    "sel4-panicking"
    "sel4-reset" # not yet implement for x86
    "sel4-newlib" # requires env
    "sel4-musl" # requires musl
    "sel4-kernel-loader" # requires env
    "tests-root-task-dafny-core"
  ];

  # TODO extract from .vscode/settings.json or workspace manifest
  workspaceDefaultMembers = lib.filterAttrs
    (_name: crate: !(lib.any (bad: lib.hasAttr bad crate.nonOptionalClosure) blockList))
    crates;

  mkEnv =
    { members
    , optionalTargetTriple ? null
    , optionalTargetDirName ? null
    , extraManifest ? {}
    }:

    let

      exclude = lib.filterAttrs
        (name: _crate: !(lib.hasAttr name members))
        crates;

      memberPaths = lib.naturalSort (lib.mapAttrsToList (_name: crate: relPath crate.path) members);
      excludePaths = lib.naturalSort (lib.mapAttrsToList (_name: crate: relPath crate.path) exclude);

      manifest = writers.writeTOML "Cargo.toml" (crateUtils.clobber [
        {
          workspace = {
            resolver = "3";
            default-members = memberPaths;
            members = memberPaths;
            exclude = excludePaths;
          };
        }
        extraManifest
      ]);

      lockfile = pruneLockfile {
        inherit (rustEnvironment) rustToolchain vendoredSuperLockfile;
        rootCrates = lib.attrValues members;
        inherit extraManifest;
      };

      configFile = writers.writeTOML "rust-toolchain.toml" (crateUtils.clobber [
        {
          unstable = {
            unstable-options = true;
          };
        }
        # TODO is this necessary?
        # (lib.optionalAttrs (optionalTargetTriple != null) (crateUtils.linkerConfig {
        #   inherit rustEnvironment;
        #   targetTriple = optionalTargetTriple;
        # }))
        (lib.optionalAttrs (optionalTargetTriple != null) {
          unstable = {
            build-std = [ "core" "alloc" "compiler_builtins" ];
            build-std-features = [
              "compiler-builtins-mem"
            ] ++ lib.optionals (rustEnvironment.compilerRTSource != null) [
              "compiler-builtins-c"
            ];
          };
          build = {
            target = targetTriple.name;
          };
          env = {
            RUST_TARGET_PATH = rustEnvironment.mkTargetPath optionalTargetTriple;
          } // lib.optionalAttrs (rustEnvironment.compilerRTSource != null) {
            "CC_${targetTriple.name}" = "${stdenv.cc.targetPrefix}gcc";
            RUST_COMPILER_RT_ROOT = rustEnvironment.compilerRTSource;
          };
        })
      ]);

      settingsFile = writers.writeJSON "settings.json" ({
        "terminal.integrated.cwd" = toString topLevelPath;
        "rust-analyzer.server.path" = "rust-analyzer";
        "rust-analyzer.cargo.allTargets" = false;
        "rust-analyzer.cargo.targetDir" = true; # use subdir of outer target-dir
      } // lib.optionalAttrs (optionalTargetTriple != null) {
        # "rust-analyzer.check.workspace" = false;
        "rust-analyzer.cargo.extraArgs" =
          lib.concatMap (name: [ "--exclude" name ]) (lib.attrNames exclude)
        ;
      });

      targetDirSuffix =
        if optionalTargetTriple == null
        then "build"
        else "host/${optionalTargetTriple.name}/${optionalTargetDirName}";

      targetDir = toString (targetDirBase + "/${targetDirSuffix}");

      workspaceNameSuffix = 
        if optionalTargetTriple == null
        then "build"
        else "host-${optionalTargetTriple.name}";

      workspaceName = "${hostPlatform.config}-${workspaceNameSuffix}-workspace";

      mkSubdir = prefix: map (v: v // { name = "${prefix}/${v.name}"; });

      dir = linkFarm workspaceName (mkSubdir workspaceName [
        { name = ".vscode/settings.json"; path = settingsFile; }
        { name = ".cargo/config.toml"; path = configFile; }
        { name = "rust-toolchain.toml"; path = toolchainFile; }
        { name = "rustfmt.toml"; path = toString (topLevelPath + "/rustfmt.toml"); }
        { name = "Cargo.toml"; path = manifest; }
        { name = "Cargo.lock"; path = lockfile; }
        { name = "crates"; path = toString (topLevelPath + "/crates"); }
        { name = "target"; path = targetDir; }
      ]);

      sh = shell.overrideAttrs (attrs: {
        shellHook = (attrs.shellHook or "") + ''
          v() {
            mkdir -p ${targetDir} && TMPDIR=/tmp code ${dir}/${workspaceName}
          }
        '';
      });

    in {
      inherit dir sh;
      # packages = lib.concatMapStrings (name: " -p ${name}") (lib.attrNames members);
    };

  union = lib.foldl' (a: b: a // b) {};

  build = mkEnv {
    members =
      let
        allProcMacroCrates = lib.filterAttrs (_: crate: crate.manifest.lib.proc-macro or false) crates;
        crateBuildDeps = crate:
          lib.mapAttrs
            (k: _: crates.${k})
            (crateUtils.getDeps.build crate)
        ;
        allBuildDepCrates = union (lib.mapAttrsToList (_: crateBuildDeps) crates);
        allBuildDepCratesProp = union (lib.mapAttrsToList (_: crate: crate.nonOptionalClosure) allBuildDepCrates);
        allProcMacroCratesProp = union (lib.mapAttrsToList (_: crate: crate.nonOptionalClosure) allProcMacroCrates);
      in
        workspaceDefaultMembers // allBuildDepCratesProp // allProcMacroCratesProp;
    extraManifest = {
      patch = globalPatchSection;
    };
  };

  requiredCrates = subCrates:
    # TODO minimal failure:
    # cargo b -p sel4-config-macros -p sel4-capdl-initializer-types
    # (union (map (crate: crate.normalNonOptionalClosureWithProcMacros) subCrates));
    (union (map (crate: crate.normalNonOptionalClosure) subCrates));

in
rec {

  inherit build;

  capdl = mkEnv {
    members = requiredCrates (with crates; [
      sel4-capdl-initializer
    ]);
    optionalTargetTriple = defaultRustTargetTriple;
    optionalTargetDirName = "capdl";
  };

}

# NOTE
# extraManifest = {
#   patch = globalPatchSection;
# };
