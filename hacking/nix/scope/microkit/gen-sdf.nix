#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand
, sdfgen
}:

{ board
}:

{ script
}:

runCommand "banscii.system" {
  nativeBuildInputs = [
    sdfgen
  ];
} ''
  python3 ${script} \
    --board ${board} \
    -o $out
''
