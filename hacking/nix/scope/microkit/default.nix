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
, vendorLockfile
, toTOMLFile
, defaultRustEnvironment
, rustEnvironment ? defaultRustEnvironment
, fenix
, libclangPath
}:

{ platformRequiresLoader, microkitConfig, ... }:

let
  inherit (microkitConfig) board config;
  inherit (pkgsBuildBuild) rustPlatform;
  inherit (rustPlatform) importCargoLock fetchCargoVendor;

  microkitSource = sources.microkit;

  kernelSource = sources.seL4;

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
    else if hostPlatform.isx86_64 then "x86_64"
    else throw "unknown arch";

  rustToolchain = fenix.fromToolchainFile {
    file = microkitSource + "/rust-toolchain.toml";
    sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
  };

  sdk =
    let
      vendoredLockfile = vendorLockfile {
        inherit rustToolchain;
        lockfile = microkitSource + "/tool/microkit/Cargo.lock";
      };

      x = importCargoLock {
        lockFile = microkitSource + "/tool/microkit/Cargo.lock";
        allowBuiltinFetchGit = true;
      };

      # y = importCargoLock {
      #   lockFile = microkitSource + "/tool/microkit/Cargo.lock";
      #   allowBuiltinFetchGit = true;
      # };

      y = importCargoLock {
        lockFile = rustToolchain + "/lib/rustlib/src/rust/library/Cargo.lock";
        allowBuiltinFetchGit = true;
      };

      z = runCommand "z" {} ''
        mkdir $out
        cp -r ${x}/* $out
        cp -r ${y}/* $out
      '';

      cargoConfigFile = toTOMLFile "config.toml" {
        source.crates-io.replace-with = "vendored-sources";
        source.vendored-sources.directory = z;
        source."git+https://github.com/nspin/rust-seL4?rev=6918a96f8bcbf7b4f655c0986089905f906701db#6918a96f8bcbf7b4f655c0986089905f906701db" = {
          git = "https://github.com/nspin/rust-seL4";
          rev = "6918a96f8bcbf7b4f655c0986089905f906701db";
          replace-with = "vendored-sources";
        };

      };
    in
      stdenv.mkDerivation {
        passthru = {
          inherit x y z;
        };
        name = "microkit-sdk";

        src = microkitSource;

        LIBCLANG_PATH = libclangPath;

        # src = lib.cleanSourceWith {
        #   src = microkitSource;
        #   filter = name: type:
        #     let baseName = baseNameOf (toString name);
        #     in !(type == "directory" && baseName == "tool");
        # };

        nativeBuildInputs = [
          cmake ninja
          dtc libxml2
          python312Packages.sel4-deps
          rustToolchain
        ];

        depsBuildBuild = [
          # NOTE: cause drv.__spliced.buildBuild to be used to work around splicing issue
          qemuForSeL4
          buildPackages.stdenv.cc
        ];

        dontFixup = true;

        configurePhase = ''
          cat ${cargoConfigFile} >> tool/microkit/.cargo/config.toml
        '';

        buildPhase = ''
          python3 build_sdk.py \
            --sel4 ${kernelSourcePatched} \
            --boards ${board} \
            --configs ${config} \
            --gcc-toolchain-prefix-${sdkArch} ${lib.removeSuffix "-" stdenv.cc.targetPrefix} \
            --skip-docs \
            --skip-tar
        '';

        installPhase = ''
          mv release/microkit-sdk-* $out
        '';
      };

  tool = sdk;

  sdkWithTool = sdk;

  outputName = if platformRequiresLoader then "loader" else "root-task";

  mkLoader =
    { systemXML
    , searchPath
    }:
    lib.fix (self: runCommand "system" {
      passthru = {
        inherit systemXML;
        image = "${self}/${outputName}.img";
      };
    } ''
      mkdir $out
      MICROKIT_SDK=${sdk} \
        ${tool}/bin/microkit ${systemXML} \
          --search-path ${lib.concatStringsSep " " searchPath} \
          --board ${board} \
          --config ${config} \
          -o $out/${outputName}.img \
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
      rootTaskImage = loader.image;
      debuggingLinks = [
        { name = "${outputName}.img"; path = "${loader}/${outputName}.img"; }
        { name = "report.txt"; path = "${loader}/report.txt"; }
        { name = "sdk/elf"; path = "${sdk}/board/${board}/${config}/elf"; }
        { name = "sel4-symbolize-backtrace";
          path = "${buildPackages.this.sel4-backtrace-cli}/bin/sel4-symbolize-backtrace";
        }
      ] ++ extraDebuggingLinks;
    } // passthru;

in rec {
  inherit
    sdk tool
    sdkWithTool
    mkSystem
  ;
}
