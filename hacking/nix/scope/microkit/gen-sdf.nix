#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ runCommand
, linkFarm
, acacia
, sources
}:

{ board
}:

{ script
}:

let
  ukitTestUtilsSrc =
    let
      name = "ukit_test_utils";
      path = sources.srcRoot + "/hacking/src/python/${name}";
    in
      linkFarm name [
        { inherit name path; }
      ];
in
runCommand "system.xml" {
  nativeBuildInputs = [
    acacia
  ];
} ''
  PYTHONPATH="${ukitTestUtilsSrc}:''${PYTHONPATH:-}" \
    python3 ${script} \
      --board ${board} \
      -o $out
''
