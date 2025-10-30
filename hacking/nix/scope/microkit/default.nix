#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPlatform, hostPlatform
, buildPackages, pkgsBuildBuild
, linkFarm, writeScript, runCommand
, buildEnv
, callPackage
, cmake, ninja
, dtc, libxml2
, python312Packages
, qemuForSeL4
, sources
, libclangPath
, vendorLockfile
, crateUtils
, defaultRustEnvironment
, defaultRustTargetTriple
, buildSysroot
, rustEnvironment ? defaultRustEnvironment
}:

{ board, config }:

let
  microkitSource = sources.microkit;

  microkitRustSeL4Source = sources.microkitRustSeL4;

  # rustToolchainAttrs = builtins.fromTOML (builtins.readFile (src + "/rust-toolchain.toml"));

  # inherit (rustToolchainAttrs.toolchain) channel;

  # rustToolchain = assembleRustToolchain {
  #   inherit channel;
  #   sha256 = "sha256-Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
  # };

  # rustEnvironment = lib.fix (self: elaborateRustEnvironment (mkDefaultElaborateRustEnvironmentArgs {
  #   inherit rustToolchain;
  # } // {
  #   inherit channel;
  #   mkCustomTargetPath = mkMkCustomTargetPathForEnvironment {
  #     rustEnvironment = self;
  #   };
  # }));

  kernelSource = sources.seL4.rust-microkit;

  kernelSourcePatched = stdenv.mkDerivation {
    name = "kernel-source-for-microkit";
    src = kernelSource;
    phases = [ "unpackPhase" "patchPhase" "installPhase" ];
    nativeBuildInputs = [
      python312Packages.sel4-deps
    ];
    postPatch = ''
      # patchShebangs can't handle env -S
      rm configs/*_verified.cmake

      patchShebangs --build .
    '';
    installPhase = ''
      cp -R ./ $out
    '';
  };

  sdkArch =
    if hostPlatform.isAarch64 then "aarch64"
    else if hostPlatform.isRiscV64 then "riscv64"
    else throw "unkown arch";

  sdkWithoutInitializer = stdenv.mkDerivation {
    name = "microkit-sdk-without-tool";

    src = lib.cleanSourceWith {
      src = microkitSource;
      filter = name: type:
        let baseName = baseNameOf (toString name);
        in !(type == "directory" && baseName == "tool");
    };

    nativeBuildInputs = [
      cmake ninja
      dtc libxml2
      python312Packages.sel4-deps
    ];

    depsBuildBuild = [
      # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
      qemuForSeL4
    ];

    dontConfigure = true;
    dontFixup = true;

    buildPhase = ''
      python3 build_sdk.py \
        --sel4 ${kernelSourcePatched} \
        --boards ${board} \
        --configs ${config} \
        --gcc-toolchain-prefix-${sdkArch} ${lib.removeSuffix "-" stdenv.cc.targetPrefix} \
        --skip-tool \
        --skip-initialiser \
        --skip-docs \
        --skip-tar
    '';

    installPhase = ''
      mv release/microkit-sdk-* $out
    '';
  };

  tool =
    let
      vendoredLockfile = vendorLockfile {
        inherit (rustEnvironment) rustToolchain;
        lockfile = microkitSource + "/tool/microkit/Cargo.lock";
      };

      cargoConfigFile = crateUtils.toTOMLFile "config.toml" vendoredLockfile.configFragment;

    in
      stdenv.mkDerivation ({
        name = "microkit-sdk-just-tool";

        src = lib.cleanSource (microkitSource + "/tool/microkit");

        nativeBuildInputs = [
          rustEnvironment.rustToolchain
        ];

        depsBuildBuild = [
          buildPackages.stdenv.cc
        ];

      } // lib.optionalAttrs (!rustEnvironment.isNightly) {
        # HACK
        RUSTC_BOOTSTRAP = 1;
      } // {

        dontInstall = true;
        dontFixup = true;

        configurePhase = ''
          cat ${cargoConfigFile} >> .cargo/config.toml
        '';

        buildPhase = ''
          cargo build -Z unstable-options --frozen --config ${cargoConfigFile} ${rustEnvironment.artifactDirFlag} $out/bin
        '';
      });

  initializer =
    let
      vendoredLockfile = vendorLockfile {
        inherit (rustEnvironment) rustToolchain;
        lockfile = microkitRustSeL4Source + "/Cargo.lock";
      };

      sysroot = buildSysroot {
        inherit rustEnvironment;
        targetTriple = defaultRustTargetTriple;
      };

      cargoConfigFile = crateUtils.toTOMLFile "config.toml" (crateUtils.clobber [
        vendoredLockfile.configFragment
        {
            target.${defaultRustTargetTriple.name}.rustflags = [
            "--sysroot" sysroot
          ];
        }
        # rustEnvironment.vendoredSysrootLockfile.configFragment
      ]);

    in
      stdenv.mkDerivation ({
        name = "microkit-sdk-just-initializer";

        src = microkitRustSeL4Source;

        nativeBuildInputs = [
          rustEnvironment.rustToolchain
        ];

        depsBuildBuild = [
          buildPackages.stdenv.cc
        ];

      } // lib.optionalAttrs (!rustEnvironment.isNightly) {
        # HACK
        RUSTC_BOOTSTRAP = 1;
      } // {

        dontInstall = true;
        dontFixup = true;

        LIBCLANG_PATH = libclangPath;
        SEL4_INCLUDE_DIRS = "${sdkWithoutInitializer}/board/${board}/${config}/include";

        buildPhase = ''
          cargo build \
            -Z unstable-options \
            --frozen \
            --config ${cargoConfigFile} \
            ${rustEnvironment.artifactDirFlag} . \
            --target ${defaultRustTargetTriple.name} \
            -p sel4-capdl-initializer

          d=$out/board/${board}/${config}/elf
          mkdir -p $d
          cp sel4-capdl-initializer.elf $d/initialiser.elf
        '';
      });

  sdk = buildEnv {
    name = "microkit-sdk-with-tool";
    paths = [
      sdkWithoutInitializer
      initializer
    ];
  };

  sdkWithTool = buildEnv {
    name = "microkit-sdk-with-tool";
    paths = [
      sdk
      tool
    ];
  };

  mkLoader =
    { systemXML
    , searchPath
    }:
    lib.fix (self: runCommand "system" {
      passthru = {
        inherit systemXML;
        image = "${self}/loader.img";
      };
    } ''
      mkdir $out
      MICROKIT_SDK=${sdk} \
        ${tool}/bin/microkit ${systemXML} \
          --search-path ${lib.concatStringsSep " " searchPath} \
          --board ${board} \
          --config ${config} \
          -o $out/loader.img \
          -r $out/report.txt
    '');

  mkSystem =
    { systemXML
    , searchPath
    , extraDebuggingLinks ? []
    , passthru ? {}
    }:
    let
      loader = mkLoader { inherit systemXML searchPath; };
    in {
      inherit loader;
      loaderImage = loader.image;
      debuggingLinks = [
        { name = "loader.img"; path = "${loader}/loader.img"; }
        { name = "report.txt"; path = "${loader}/report.txt"; }
        { name = "sdk/elf"; path = "${sdk}/board/${board}/${config}/elf"; }
        { name = "sel4-symbolize-backtrace";
          path = "${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace";
        }
      ] ++ extraDebuggingLinks;
    } // passthru;

in rec {
  inherit
    sdk
    tool
    initializer
    sdkWithTool
    mkSystem
  ;
}
