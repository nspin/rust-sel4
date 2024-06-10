{ lib, stdenv
, buildPackages
, fetchurl, fetchpatch
, python3Packages
, pkg-config, ninja, meson, perl
, zlib, lzo, glib
, bison, flex, dtc
, pixman, vde2
, texinfo
, snappy, libaio, libtasn1, gnutls, nettle, curl
, attr, libcap, libcap_ng, libslirp
, hostCpuTargets ? []
}:

stdenv.mkDerivation rec {
  pname = "qemu";
  version = "9.0.0"; # n
  # version = "8.2.0"; # n
  # version = "8.1.0"; # n
  # version = "8.1.0-rc1"; # n
  # version = "8.1.0-rc0"; # ?
  # version = "8.0.5"; # y
  # version = "8.0.4"; # y
  # version = "8.0.3"; # y
  # version = "8.0.0"; # y

  src = fetchurl {
    url = "https://download.qemu.org/qemu-${version}.tar.xz";
    hash = "sha256-MnCKxmww2MiSYz6paMdxwcdtWX1w3erSGg0izPOG2mk="; # 9.0.0
    # hash = "sha256-vwDS+hIBDfiwrekzcd71jmMssypr/cX1oP+Oah+xvzI="; # 8.2.0
    # hash = "sha256-cQwQEZjjNNR2Lu9l9km8Q/qKXddTA1VLis/sPrJfDlU="; # 8.1.0
    # hash = "sha256-0v7c+WLqybeeDeno4Bun9jCVw4fHDZxTZLLMzJMmHC0="; # 8.1.0-rc1
    # hash = "sha256-uQX+PRzDpvHNYMvUgKB7bJ7LbX9plCWIS5DeQwcBGIw="; # 8.1.0-rc0
    # hash = "sha256-kdMCTVHkQcI13LGwyHyzqrMCKDFm6NPV+CgqoGw0a+E="; # 8.0.5
    # hash = "sha256-gcgX3aOK+Vi+W+8abPVbZYuy0/uHwealcd5reyxEUWw="; # 8.0.4
    # hash = "sha256-7PTTLL7505e/yMxQ5NHpKhswJTvzLo7nPHqNz5ojKwk="; # 8.0.3
    # hash = "sha256-u2DwNBUxGB1sw5ad0ZoBPQQnqH+RgZOXDZrbkRMeVtA="; # 8.0.0
  };

  depsBuildBuild = [
    buildPackages.stdenv.cc
  ];

  nativeBuildInputs = [
    pkg-config meson ninja
    bison flex dtc
    perl

    # Don't change this to python3 and python3.pkgs.*, breaks cross-compilation
    python3Packages.python
    python3Packages.sphinx
    python3Packages.sphinx-rtd-theme
  ];

  buildInputs = [
    dtc zlib lzo glib pixman vde2 texinfo
    snappy libtasn1 gnutls nettle curl libslirp
    libaio libcap_ng libcap attr
  ];

  dontUseMesonConfigure = true; # meson's configurePhase isn't compatible with qemu build

  patches = [
    # nspin/arm-virt-sp804
    (fetchurl {
      url = "https://github.com/coliasgroup/qemu/commit/7994b0d17da7dbf1cf2da3e6555914e23559b23e.patch";
      sha256 = "sha256-o5Z1LYF6pwqBGrP4AYOIXmhSg75w7mIRuxvj2ZCO+HY=";
    })
    # nspin/opensbi-fw-payload-use-elf-entry-point
    (fetchurl {
      url = "https://github.com/coliasgroup/qemu/commit/db69d0a7dc0af9d8130328347fdd81ab5fa9e352.patch";
      sha256 = "sha256-12uGZRO6T1uWYvblAx5/FdRsuZZ1B1iWT9ZxpN3Qga0=";
    })
    # ../../../../tmp/1.diff
    ../../../../tmp/2.diff
  ];

  postPatch = ''
    # Otherwise tries to ensure /var/run exists.
    sed -i "/install_emptydir(get_option('localstatedir') \/ 'run')/d" \
        qga/meson.build

    cp ${../../../../tmp/1.2-opensbi-riscv64-generic-fw_dynamic.bin} pc-bios/opensbi-riscv64-generic-fw_dynamic.bin
  '';

  preConfigure = ''
    unset CPP # intereferes with dependency calculation
    # this script isn't marked as executable b/c it's indirectly used by meson. Needed to patch its shebang
    chmod +x ./scripts/shaderinclude.py
    patchShebangs .
    # avoid conflicts with libc++ include for <version>
    mv VERSION QEMU_VERSION
    substituteInPlace configure \
      --replace '$source_path/VERSION' '$source_path/QEMU_VERSION'
    substituteInPlace meson.build \
      --replace "'VERSION'" "'QEMU_VERSION'"
  '';

  configureFlags = [
    "--localstatedir=/var"
    "--sysconfdir=/etc"
    "--enable-linux-aio"
    "--enable-slirp"
    "--cross-prefix=${stdenv.cc.targetPrefix}"
    "--target-list=${lib.concatStringsSep "," hostCpuTargets}"
  ];

  preBuild = ''
    cd build
  '';
}
