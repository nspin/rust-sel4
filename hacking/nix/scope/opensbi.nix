#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib
, stdenv
, fetchFromGitHub
, python3
, sources
}:

stdenv.mkDerivation rec {
  pname = "opensbi";
  # version = "1.4";
  version = "1.3.1";

  # src = fetchFromGitHub {
  #   owner = "riscv-software-src";
  #   repo = "opensbi";
  #   rev = "v${version}";
  #   # hash = "sha256-T8ZeAzjM9aeTXitjE7s+m+jjGGtDo2jK1qO5EuKiVLU="; # 1.4
  #   hash = "sha256-JNkPvmKYd5xbGB2lsZKWrpI6rBIckWbkLYu98bw7+QY="; # 1.3.1
  # };

  src = lib.cleanSource (sources.localRoot + "/opensbi");

  postPatch = ''
    patchShebangs ./scripts
  '';

  nativeBuildInputs = [ python3 ];

  makeFlags = [
    "PLATFORM=generic"
  ];

  installFlags = [
    "I=$(out)"
  ];

  dontFixup = true;
}
