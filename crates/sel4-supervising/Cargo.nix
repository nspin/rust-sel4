#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates }:

mk {
  package.name = "sel4-supervising";
  dependencies = {
    inherit (localCrates) sel4 sel4-supervising-types;
  };
}
