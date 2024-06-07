#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "banscii-uart-driver";
  dependencies = {
    inherit (versions) heapless;
    inherit (localCrates)
      sel4-microkit-message
      banscii-uart-driver-traits
      banscii-uart-driver-interface-types
    ;
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; };
    banscii-pl011-driver = localCrates.banscii-pl011-driver // { optional = true; };
    banscii-xuartps-driver = localCrates.banscii-xuartps-driver // { optional = true; };
  };
  features = {
    board-qemu_virt_aarch64 = [ "banscii-pl011-driver" ];
    board-zcu102 = [ "banscii-xuartps-driver" ];
  };
}
