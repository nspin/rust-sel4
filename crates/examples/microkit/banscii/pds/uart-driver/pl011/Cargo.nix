#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates }:

mk {
  package.name = "banscii-pl011-driver";
  dependencies = {
    inherit (versions) tock-registers;
    inherit (localCrates) banscii-uart-driver-traits;
  };
}
