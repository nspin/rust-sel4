#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, verusSource }:

mk {
  package.name = "sel4-bitfield-ops-verified";
  dependencies = {
    inherit (localCrates) sel4-bitfield-ops;
    builtin = verusSource;
    builtin_macros = verusSource;
    vstd = verusSource // { default-features = false; };
  };
  package.metadata.verus = {
    verify = true;
  };
}
