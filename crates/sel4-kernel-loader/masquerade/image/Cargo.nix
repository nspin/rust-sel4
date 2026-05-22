#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-kernel-loader-masquerade-image";
  dependencies = {
    inherit (localCrates) sel4-no-panic;
  };
}
