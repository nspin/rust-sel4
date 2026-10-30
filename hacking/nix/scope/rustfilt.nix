#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ rustPlatform
, fetchCrate
}:

rustPlatform.buildRustPackage rec {
  pname = "rustfilt";
  version = "0.2.2";

  src = fetchCrate {
    inherit pname version;
    hash = "sha256-bQWLygOq9sTUUZECYgjWMXmObP/EnPYn3O9noKQRNeQ=";
  };

  cargoHash = "sha256-Eo4ICkuWtr2GpjX85ZvBzQA+Qgq0ioIID8+pfyV/vSM=";
}
