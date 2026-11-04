#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions, smoltcpWith }:

mk {
  package.name = "sddf-smoltcp";
  dependencies = {
    inherit (versions) heapless;
    smoltcp = smoltcpWith [];
    inherit (localCrates)
      sddf
      sel4-shared-memory
    ;
  };
}
