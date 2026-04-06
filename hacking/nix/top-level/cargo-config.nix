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
  inherit (build) writers linkFarm writeShellApplication this llvm python312;
  inherit (this) sources crateUtils capdl-tool sdfgen;

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

  # TODO depend on whether -musl
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

  capdlInitializerTargetForTarget = target: {
    "aarch64" = "aarch64-sel4-roottask-minimal";
    "armv7" = "armv7a-sel4-roottask-minimal";
    "armv7a" = "armv7a-sel4-roottask-minimal";
    "riscv64gc" = "riscv64gc-sel4-roottask-minimal";
    "riscv64imac" = "riscv64imac-sel4-roottask-minimal";
    "riscv32imac" = "riscv32imac-sel4-roottask-minimal";
    "riscv32imafc" = "riscv32imafc-sel4-roottask-minimal";
    "x86_64" = "x86_64-sel4-roottask-minimal";
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
          if !(hasMusl target)
          then {
            env = {
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg (getNewlibDir stdenv); # TODO necessary?
            };
          }
          else if hasSeL4 target
          then {
            env = {
              "CFLAGS_${target}" = "-nostdlib ${mkIncludeArg muslForSeL4}"; # TODO necessary?
              "BINDGEN_EXTRA_CLANG_ARGS_${target}" = mkIncludeArg muslForSeL4; # TODO necessary?
            };
            target.${target} = {
              rustflags = [
                "-L${muslForSeL4}/lib"
                "-L${dummyLibunwind}/lib"
              ];
            };
          }
          # special case, not included in rustup
          else if target == "riscv32gc-unknown-linux-musl"
          then
            let
              thesePkgs = pkgs.host.riscv32.gc.linuxMusl;
            in {
              target.${target} = {
                rustflags = [
                  "-L${thesePkgs.stdenv.cc.cc}/lib/gcc/${thesePkgs.stdenv.hostPlatform.config}/${thesePkgs.stdenv.cc.version}"
                  "-L${thesePkgs.stdenv.cc.libc}/lib"
                  "-L${thesePkgs.libunwind}/lib"
                ];
              };
            }
          else {
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

  mkRunner = body: ''
    set +x

    external_exe="$1"
    shift

    target_dir="$WORLD_TARGET_DIR"
    simulate_script="$WORLD_QEMU_SCRIPT"

    parent="$target_dir/runner"
    mkdir -p "$parent"
    d="$(mktemp -d --tmpdir="$parent")"

    echo 'd:' >&2
    echo "$d" >&2

    cleanup() {
      rm -rf "$d"
    }

    # trap cleanup EXIT

    exe_name="$(basename "$external_exe")"
    exe="$d/$exe_name"

    cp "$external_exe" "$exe"

    ${body}

    cargo run -p sel4-test-sentinels-wrapper -- "$simulate_script" "$image" "$@"

    stty echo
  '';

  rootTaskRunner = target: writeShellApplication {
    name = "root-task-runner";
    runtimeInputs = [
    ];
    checkPhase = "";
    text = mkRunner (if firstSegment target == "x86_64" then ''
      image="$exe"
    '' else ''
      image="$d/image.elf"

      cargo build \
        --config ${byTarget.${loaderTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-kernel-loader \
        --artifact-dir "$d"

      cargo run -p sel4-kernel-loader-add-payload -- \
        --loader "$d/sel4-kernel-loader" \
        --sel4-prefix "$SEL4_PREFIX" \
        --app "$exe" \
        -o "$image"
    '');
  };

  microkitRunner = writeShellApplication {
    name = "microkit-runner";
    runtimeInputs = [
      llvm
      (python312.withPackages (_: [
        sdfgen
      ]))
    ];
    text = mkRunner ''
      export PYTHONPATH="${toString ../../src/python}:''${PYTHONPATH:-}"

      (
        llvm-objcopy --dump-section .sdf_xml="$d/system.xml" "$exe" 2>/dev/null
      ) || (
        llvm-objcopy --dump-section .sdf_script="$d/system.py" "$exe"; \
        python3 "$d/system.py" \
            --board "$MICROKIT_BOARD" \
            -o "$d/system.xml"
      )

      image="$d/image.elf"

      "$MICROKIT_SDK/bin/microkit" "$d/system.xml" \
        --search-path "$d" \
        --board "$MICROKIT_BOARD" \
        --config "$MICROKIT_CONFIG" \
        -o "$image" \
        -r "$d/report.txt"
    '';
  };

  testfwRunner = target: writeShellApplication {
    name = "testfw-runner";
    excludeShellChecks = [
      "SC2317"
      "SC2329"
      "SC2154"
    ];
    runtimeInputs = [
      llvm
      capdl-tool
      (python312.withPackages (p: with p; [
        future six
        aenum sortedcontainers
        pyyaml pyelftools pyfdt
      ]))
    ];
    text = mkRunner (''
      export PYTHONPATH="${toString ../../src/python}:${sources.pythonCapDLTool}:''${PYTHONPATH:-}"

      llvm-objcopy --dump-section .capdl_script="$d/system.py" "$exe"

      script_out_dir="$d/cdl"

      python3 "$d/system.py" \
        --search-path "$d" \
        --object-sizes "$WORLD_OBJECT_SIZES" \
        -o "$script_out_dir"

      parse-capDL --object-sizes="$WORLD_OBJECT_SIZES" --json="$d/cdl.json" "$script_out_dir/spec.cdl"

      image="$d/image.elf"
      root_task="$d/root-task.elf"

      cargo build \
        --config ${byTarget.${capdlInitializerTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-capdl-initializer \
        --artifact-dir "$d"

      cargo run -p sel4-capdl-initializer-add-spec -- \
        -v \
        -e "$d/sel4-capdl-initializer.elf" \
        -f "$d/cdl.json" \
        -d "$script_out_dir/links" \
        --object-names-level 2 \
        --no-embed-frames \
        --no-deflate \
        -o "$root_task"
    '' + (if firstSegment target == "x86_64" then ''
      image="$root_task"
    '' else ''
      image="$d/image.elf"

      cargo build \
        --config ${byTarget.${loaderTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-kernel-loader \
        --artifact-dir "$d"

      cargo run -p sel4-kernel-loader-add-payload -- \
        --loader "$d/sel4-kernel-loader" \
        --sel4-prefix "$SEL4_PREFIX" \
        --app "$root_task" \
        -o "$image"
    ''));
  };

  byTarget = lib.genAttrs allTargets (target:
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
    ]))
  );

  byTargetLinks = linkFarm "by-target" (lib.flip lib.mapAttrs' byTarget (k: v: lib.nameValuePair ("${k}.toml") v));

  worlds = lib.mapAttrs
    (_: attrs: attrs.none.this.worlds or attrs.default.none.this.worlds /* HACK */)
    (lib.filterAttrs (n: _: n != "ia32") pkgs.host)
  ;

  configForWorld = attrPath: world:
    let
      targetDir = "${targetRootDir}/by-world/${lib.concatStringsSep "." attrPath}";
      simulateScript =
        let script = world.simulateScript;
        in "${script}/bin/${script.name}";
      runner = [
        "cargo" "run" "-p" "sel4-test-runner" "--"
        "--target-dir" targetDir
        "--object-sizes" world.objectSizes
        "--sel4-kernel-config" world.seL4KernelConfigFile
      ] ++ lib.optionals world.worldConfig.canSimulate [
        "--simulate-script" simulateScript
      ] ++ lib.optionals world.worldConfig.isMicrokit [
        "--microkit-sdk" world.microkit.sdk
        "--microkit-board" world.worldConfig.microkitConfig.board
        "--microkit-config" world.worldConfig.microkitConfig.config
      ];
    in {
      build.target-dir = targetDir;
      env = world.seL4RustEnvVars;
      target."cfg(all())".runner = runner;
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
