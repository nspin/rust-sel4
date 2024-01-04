#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, localCrates, versions }:

mk {
  package.name = "sel4-newlib";
  features = {
    default = [ "detect-libc" ];
    detect-libc = [];
    nosys = [];
    _exit = [ "sel4-panicking-env" ];
    _write = [ "sel4-panicking-env" ];
    all-symbols = [
      "_exit"
      "_write"
    ];
  };
  dependencies = {
    log = { version = versions.log; optional = true; };
    sel4-panicking-env = localCrates.sel4-panicking-env // { optional = true; };
  };
  build-dependencies = {
    cc = "1.0.82";
  };
}
