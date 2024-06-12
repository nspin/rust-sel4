#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "banscii-serial-driver";
  dependencies = {
    inherit (localCrates)
      sel4-microkit-message
      sel4-microkit-embedded-hal-adapters
      
    ;
    sel4-microkit = localCrates.sel4-microkit // { default-features = false; };
    sel4-pl011-driver = localCrates.sel4-pl011-driver // { optional = true; };
    sel4-xuartps-driver = localCrates.sel4-xuartps-driver // { optional = true; };
  };
  features = {
    board-qemu_virt_aarch64 = [ "sel4-pl011-driver" ];
    board-zcu102 = [ "sel4-xuartps-driver" ];
  };
}
