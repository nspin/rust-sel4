#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, pkgs }:

let
  mk = { hostPkgs, mcs }:
    let
      inherit (hostPkgs) hostPlatform;
      configName = "${config.KernelSeL4Arch.value}-${if mcs then "mcs" else "legacy"}";
      fileNamePrefix = "seL4-prefix-${configName}";
      fileName = "${fileNamePrefix}.tar.gz";
      config = with hostPkgs.this.cmakeConfigHelpers; ({
        KernelDebugBuild = on;
        KernelPrinting = on;
        KernelVerificationBuild = off;
      } // lib.optionalAttrs mcs {
        KernelIsMCS = on;
      } // lib.optionalAttrs hostPlatform.isAarch {
        KernelPlatform = mkString "qemu-arm-virt";
        KernelSeL4Arch = mkString (if hostPlatform.is64bit then "aarch64" else "aarch32");
      } // lib.optionalAttrs hostPlatform.isRiscV {
        KernelPlatform = mkString "qemu-riscv-virt";
        KernelSeL4Arch = mkString (if hostPlatform.is64bit then "riscv64" else "riscv32");
      } // lib.optionalAttrs hostPlatform.isx86 {
        KernelPlatform = mkString "pc99";
        KernelSeL4Arch = mkString (if hostPlatform.is64bit then "x86_64" else "ia32");
      });
      seL4 = hostPkgs.this.mkSeL4 {
        src = hostPkgs.this.sources.seL4.upstream;
        inherit config;
      };
      tarball = hostPkgs.runCommand fileName {} ''
        cp -rL --no-preserve owner,mode ${seL4} ${fileNamePrefix}
        rm -rf ${fileNamePrefix}/bin ${fileNamePrefix}/libsel4/src ${fileNamePrefix}/support/*.dtb
        tar cfJ $out ${fileNamePrefix}
      '';
    in
      lib.nameValuePair configName {
        inherit fileName tarball;
      };

  hostPkgSets = [
    pkgs.host.aarch64.none
    # pkgs.host.aarch32.none
    # pkgs.host.riscv64.default.none
    # pkgs.host.riscv32.default.none
    # pkgs.host.x86_64.none
  ];

  prefixesAttrs = lib.listToAttrs (lib.concatLists (lib.forEach hostPkgSets (hostPkgs:
    lib.forEach [ true false ] (mcs:
      mk {
        inherit hostPkgs mcs;
      }
    )
  )));

  prefixesLinks = pkgs.build.linkFarm "prefixes" (lib.mapAttrs' (_k: v: lib.nameValuePair v.fileName v.tarball) prefixesAttrs);

in {
  inherit
    prefixesAttrs
    prefixesLinks
  ;
}
