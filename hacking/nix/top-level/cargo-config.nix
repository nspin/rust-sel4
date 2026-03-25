#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ topLevel
}:

let
  inherit (topLevel) lib pkgs;
  inherit (pkgs) build;
  inherit (build) writers linkFarm writeShellApplication this;
  inherit (this) crateUtils;

  targetRootDir = toString ../../../target;

  toolchainPath = ../../../rust-toolchain.toml;

  rustupTargets = (lib.importTOML toolchainPath).toolchain.targets;

  isRustupTarget = targetName: lib.elem targetName rustupTargets;

  targetsPath = ../../../support/targets;

  customTargets =
    let
      parseTargetName = fname:
        let
          m = builtins.match "(.*)\\.json" fname;
        in
          if m == null then null else lib.elemAt m 0
      ;
      targetNames = lib.filter (x: x != null) (map parseTargetName (lib.attrNames (builtins.readDir targetsPath)));
    in
      targetNames
    ;

  hasSegment = seg: targetName: lib.elem seg (lib.splitString "-" targetName);

  firstSegment = targetName: lib.head (lib.splitString "-" targetName);

  getCCExePath = stdenv:
    let
      inherit (stdenv) cc;
    in
      "${cc}/bin/${cc.targetPrefix}gcc";

  getNewlibDir = stdenv: "${stdenv.cc.libc}/${stdenv.hostPlatform.config}";

  mkIncludeArg = d: "-I${d}/include";

  builtinMuslTargets = [
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-musleabi"
    "riscv64gc-unknown-linux-musl"
    "riscv32gc-unknown-linux-musl"
  ];

  builtinBareMetalTargets = [
    "x86_64-unknown-none"
    "aarch64-unknown-none"
    "armv7a-none-eabi"
    "riscv64imac-unknown-none-elf"
    "riscv64gc-unknown-none-elf"
    "riscv32imac-unknown-none-elf"
    "riscv32imafc-unknown-none-elf"
  ];

  allTargets = builtinMuslTargets ++ builtinBareMetalTargets ++ customTargets;

  hasMusl = hasSegment "musl";

  hasSeL4 = targetName: hasSegment "sel4" targetName || hasSegment "microkit" targetName;

  getPkgsForTarget = target: {
    "x86_64" = pkgs.host.x86_64.none;
    "aarch64" = pkgs.host.aarch64.none;
    "armv7" = pkgs.host.aarch32.none;
    "armv7a" = pkgs.host.aarch32.none;
    "riscv64gc" = pkgs.host.riscv64.gc.none;
    "riscv64imac" = pkgs.host.riscv64.imac.none;
    "riscv32gc" = pkgs.host.riscv32.gc.none;
    "riscv32imac" = pkgs.host.riscv32.imac.none;
    "riscv32imafc" = pkgs.host.riscv32.imafc.none;
  }.${firstSegment target};

  loaderTargetForTarget = target: {
    "aarch64" = "aarch64-unknown-none";
    "armv7" = "armv7a-none-eabi";
    "armv7a" = "armv7a-none-eabi";
    "riscv64gc" = "riscv64gc-unknown-none-elf";
    "riscv64imac" = "riscv64imac-unknown-none-elf";
    "riscv32imac" = "riscv32imac-unknown-none-elf";
    "riscv32imafc" = "riscv32imafc-unknown-none-elf";
  }.${firstSegment target};

  # getQEMUSuffixForTarget = target: {
  #   "x86_64" = "x86_64";
  #   "aarch64" = "aarch64";
  #   "armv7" = "arm";
  #   "armv7a" = "arm";
  #   "riscv64gc" = "riscv64";
  #   "riscv64imac" = "riscv64";
  #   "riscv32gc" = "riscv64";
  #   "riscv32imac" = "riscv32";
  #   "riscv32imafc" = "riscv32";
  # }.${firstSegment target};

  ccConfigForTarget = target:
    let
      hostPkgs = getPkgsForTarget target;
      inherit (hostPkgs) stdenv;
      inherit (hostPkgs.this) muslForSeL4 dummyLibunwind;
    in
      crateUtils.clobber ([
        {
          env = {
            "CC_${target}" = getCCExePath stdenv;
          };
        }
        (
          if hasSeL4 target && hasMusl target
          then {
            env = {
              "CFLAGS_${target}" = "-nostdlib ${mkIncludeArg muslForSeL4}";
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg muslForSeL4;
            };
            target.${target} = {
              rustflags = [
                "-L${muslForSeL4}/lib"
                "-L${dummyLibunwind}/lib"
              ];
            };
          }
          else {
            env = {
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg (getNewlibDir stdenv);
            };
          }
        )
      ])
    ;

  ccConfigCommon = {
    env = {
      HOST_CC = getCCExePath build.stdenv;
      LIBCLANG_PATH = build.this.libclangPath;
    };
  };

  cc = writers.writeTOML "cc.toml" (crateUtils.clobber ([
    ccConfigCommon
  ] ++ map ccConfigForTarget allTargets));

  rootTaskRunner = target: writeShellApplication {
    name = "root-task-runner";
    runtimeInputs = [
    ];
    checkPhase = "";
    text = ''
      set +x

      root_task="$1"
      shift

      target_dir="$WORLD_TARGET_DIR"
      simulate_script="$WORLD_QEMU_SCRIPT"

      parent="$target_dir/runner/root-task"
      mkdir -p "$parent"
      d="$(mktemp -d --tmpdir="$parent")"

      cleanup() {
        rm -rf "$d"
      }

      # trap cleanup EXIT
    '' + (if firstSegment target == "x86_64" then ''
    '' else ''
      cargo build \
        --config ${byTarget.${loaderTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-kernel-loader \
        --artifact-dir "$d"

      cargo run -p sel4-kernel-loader-add-payload -- \
        --loader "$d/sel4-kernel-loader" \
        --sel4-prefix "$SEL4_PREFIX" \
        --app "$root_task" \
        -o "$d/image.elf"

      "$simulate_script" "$d/image.elf" "$@"
    '');
  };

  microkitRunner = writeShellApplication {
    name = "microkit-runner";
    runtimeInputs = [
    ];
    text = ''
      echo "running:" "$@"
    '';
  };

  testfwRunner = writeShellApplication {
    name = "testfw-runner";
    runtimeInputs = [
    ];
    text = ''
      echo "running:" "$@"
    '';
  };

  byTarget = lib.genAttrs allTargets (target:
    let
      mkRunner = script: {
        target.${target}.runner = "${script}/bin/${script.name}";
      };
    in
    writers.writeTOML "${target}.toml" (crateUtils.clobber ([
      {
        build = {
          target = target;
        };
      }
      (lib.optionalAttrs (!(isRustupTarget target)) {
        unstable = {
          build-std = [ "compiler_builtins" "core" "alloc" ] ++ lib.optional (hasMusl target) "std";
          build-std-features = [ "compiler-builtins-mem" ];
        };
      })
      ccConfigCommon
      (ccConfigForTarget target)
      (
        if hasSegment "roottask" target
        then mkRunner (rootTaskRunner target)
        else if hasSegment "microkit" target
        then mkRunner microkitRunner
        else if hasSegment "testfw" target # HACK
        then mkRunner testfwRunner
        else {}
      )
    ]))
  );

  byTargetLinks = linkFarm "by-target" (lib.flip lib.mapAttrs' byTarget (k: v: lib.nameValuePair ("${k}.toml") v));

  worlds = lib.mapAttrs
    (_: attrs: attrs.none.this.worlds or attrs.default.none.this.worlds /* HACK */)
    (lib.filterAttrs (n: _: n != "ia32") pkgs.host)
  ;

  mkQEMUScript = world: writeShellApplication {
    name = "simulate";
    runtimeInputs = [
    ];
    checkPhase = "";
    text = ''
      image="$1"
      shift
      exec ${lib.concatStringsSep " " (world.worldConfig.mkQEMUCmd ''"$image"'')} "$@"
    '';
  };

  configForWorld = attrPath: world:
    let
      targetDir = "${targetRootDir}/by-world/${lib.concatStringsSep "." attrPath}";
    in {
      build.target-dir = targetDir;
      env = world.seL4RustEnvVars // {
        WORLD_TARGET_DIR = targetDir;
      } // lib.optionalAttrs world.worldConfig.canSimulate {
        WORLD_QEMU_SCRIPT =
          let script = mkQEMUScript world;
          in "${script}/bin/${script.name}";
      };
    };

  byWorldList = lib.mapAttrsToListRecursiveCond
    (_: attrs: !(attrs.__isWorld or false))
    (attrPath: world: {
      path = attrPath;
      config = configForWorld attrPath world;
    })
    worlds
  ;

  byWorldLinks = linkFarm "by-world"
    (lib.listToAttrs
      (map
        ({ path, config }:
          let
            name = "${lib.concatStringsSep "." path}.toml";
          in
            lib.nameValuePair
              name
              (writers.writeTOML name config))
        byWorldList));

  links = linkFarm "generated-config" {
    "cc.toml" = cc;
    "by-target" = byTargetLinks;
    "by-world" = byWorldLinks;
  };

in {

  inherit links cc;

  utils = {
    inherit
      firstSegment
      hasSegment
      customTargets
      builtinBareMetalTargets
    ;
  };
}

# -nostdlib
