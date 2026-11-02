#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

# lib_sddf_lwip.a is not built, because it is configured by each client's lwipopts.h.

{ lib
, stdenv
, buildPackages
, symlinkJoin

, perl

, sources
, microkit
, worldConfig
}:

let
  llvm = buildPackages.llvmPackages_18;

  # see sddf/flake.nix
  clang = symlinkJoin {
    name = "clang-complete";
    paths = llvm.clang-unwrapped.all;
    postBuild = ''
      cp --remove-destination -- ${llvm.clang-unwrapped}/bin/* $out/bin/
    '';
  };

  inherit (worldConfig.microkitConfig) board config;

  src = sources.sddf;

in
stdenv.mkDerivation (finalAttrs: {
  name = "sddf-${board}-${config}";

  depsBuildBuild = [
    perl # for shasum
  ];

  nativeBuildInputs = [
    clang
    llvm.lld
    llvm.libllvm
  ];

  dontUnpack = true;

  makefile = ./Makefile;

  makeFlags = [
    "SDDF=${src}"
    "MICROKIT_SDK=${microkit.sdk}"
    "MICROKIT_BOARD=${board}"
    "MICROKIT_CONFIG=${config}"
  ];

  installFlags = [
    "PREFIX=${placeholder "out"}"
  ];

  # HACK
  # Make libmicrokit's debug printing available on its own, for programs (e.g. Rust protection
  # domains) which link against sDDF's libraries but not against libmicrokit.
  postInstall = ''
    mkdir libmicrokit_dbg
    llvm-ar x --output libmicrokit_dbg ${microkit.sdk}/board/${board}/${config}/lib/libmicrokit.a dbg.o
    llvm-ar crs $out/lib/libmicrokit_dbg.a libmicrokit_dbg/dbg.o
  '';

  enableParallelBuilding = true;

  passthru = {
    inherit src;
    includeDirs = [
      "${finalAttrs.finalPackage}/include"
      "${finalAttrs.finalPackage}/include/sddf/util/custom_libc"
    ];
  };
})
