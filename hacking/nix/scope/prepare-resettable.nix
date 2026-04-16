#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, runCommand
, sel4-reset-cli2
}:

elf:

runCommand "elf" {
  nativeBuildInputs = [
    sel4-reset-cli2
  ];
} ''
  sel4-reset-cli2 ${elf} -o $out
''
