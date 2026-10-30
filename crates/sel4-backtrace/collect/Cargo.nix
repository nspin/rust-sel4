#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, unwindingWith }:

mk {
  package.name = "sel4-backtrace-collect";
  dependencies = {
    unwinding = unwindingWith [] // { optional = true; };
  };
  features = {
    unwinding = [
      "dep:unwinding"
    ];
  };
}
