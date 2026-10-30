#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv, buildPackages
, mkShell

, pkg-config
, git
, cacert
, rustup, rustPlatform
, perl
, python3Packages
, cmake
, qemu

, kani
, verus

, strace
, cntr
, cachix
, rustfilt

, openssl

, shellForMakefile
}:

mkShell (shellForMakefile.apply {

  depsBuildBuild = [
    buildPackages.stdenv.cc
    qemu
  ];

  nativeBuildInputs = [
    pkg-config
    git
    cacert
    rustup
    perl
    cmake
    rustPlatform.bindgenHook
    strace
    cntr
    cachix
    rustfilt
  ] ++ lib.optionals stdenv.hostPlatform.isx86_64 [
    kani
    verus
  ];

  buildInputs = [
    openssl
  ];

  shellHook = ''
    kargo() {
      cargo +${kani.rustEnvironment.channel} "$@"
    }

    vargo() {
      cargo +${verus.rustEnvironment.channel} "$@"
    }
  '';
})
