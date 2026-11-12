#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, buildPackages
, callPackage
, runCommand, writeScript, linkFarm
, fetchgit
, perl
, microkit
, acacia
, llvm

, crates
, globalPatchSection
, crateUtils
, sources

, seL4Modifications
, worldConfig
, callPlatform
, mkSeL4RustTargetTriple

, canSimulate
, mkPD
}:

let
  inherit (worldConfig) isMicrokit;

  mkDiskImage = callPackage ./mk-disk-image.nix {};

  filter = { src, maxIndividualFileSize ? null, excludePatterns ? null }:
    runCommand "filtered" {} ''
      cp -r --no-preserve=owner,mode ${src} $out
      find $out -name  -delete
      ${lib.optionalString (excludePatterns != null) ''
        find $out -type f \( ${
          lib.concatMapStringsSep " -o " (pat: "-name '${pat}'") excludePatterns
        } \) -delete
      ''}
      ${lib.optionalString (maxIndividualFileSize != null) ''
        find $out -size +${maxIndividualFileSize} -delete
      ''}
    '';

  rawContent = builtins.fetchGit {
    url = "https://github.com/seL4/website_pr_hosting";
    ref = "gh-pages";
    rev = "e320919be95968dc13e2122d97737b02a8a31997";
  };

  # HACK link structure requires website_pr_hosting/${contentSubdir}
  patchedContent =
    let
      contentSubdir = "PR_371";
    in
      runCommand "patched" {} ''
        cp -r --no-preserve=owner,mode ${rawContent}/${contentSubdir} $out
        mkdir -p $out/website_pr_hosting
        cp -r --no-preserve=owner,mode ${rawContent}/${contentSubdir} $out/website_pr_hosting/${contentSubdir}/
      '';

  mkThisDiskImage = filterArgs: mkDiskImage {
    src = filter ({
      src = patchedContent;
    } // filterArgs);
  };

  commonExcludePatterns = [ ''*\?'' ];

  largeDiskImage = mkThisDiskImage { excludePatterns = commonExcludePatterns; };
  smallDiskImage = mkThisDiskImage { excludePatterns = commonExcludePatterns ++ [ "*.mp4" "*.pdf" ]; };

  # diskImage = largeDiskImage;
  diskImage = smallDiskImage;

  # Acacia reads the layouts of sDDF configuration structures from the DWARF of each protection
  # domain. The configuration sections are defined in each protection domain's root crate.
  mkPDWithConfigs = args: mkPD (args // {
    extraProfile = crateUtils.clobber [
      (args.extraProfile or {})
      {
        strip = false;
        package.${args.rootCrate.name}.debug = true;
      }
    ];
  });

  pds = {
    http-server = mkPDWithConfigs {
      rootCrate = crates.microkit-http-server-example-server;
      release = true;
      # layers = [
      #   crateUtils.defaultIntermediateLayer
      #   {
      #     crates = [
      #       "sel4-microkit"
      #     ];
      #     modifications = seL4Modifications;
      #   }
      # ];
      commonModifications = {
        modifyManifest = lib.flip crateUtils.combineConfig {
          patch.crates-io = {
            inherit (globalPatchSection.crates-io)
              ring
              # HACK
              hashbrown
            ;
          };
        };
        modifyDerivation = drv: drv.overrideAttrs (self: super: {
          HOST_CC = "${buildPackages.stdenv.cc.targetPrefix}gcc";
          nativeBuildInputs = (super.nativeBuildInputs or []) ++ [
            perl
          ];
        });
      };
      # lastLayerModifications = seL4Modifications;
    };
    pl031-driver = mkPDWithConfigs {
      rootCrate = crates.microkit-http-server-example-pl031-driver;
      release = true;
    };
    sp804-driver = mkPDWithConfigs {
      rootCrate = crates.microkit-http-server-example-sp804-driver;
      release = true;
    };
    virtio-net-driver = mkPDWithConfigs {
      rootCrate = crates.microkit-http-server-example-virtio-net-driver;
      release = true;
    };
    virtio-blk-driver = mkPDWithConfigs {
      rootCrate = crates.microkit-http-server-example-virtio-blk-driver;
      release = true;
    };
  };

  generated = runCommand "http-server-system" {
    nativeBuildInputs = [
      llvm
      acacia
    ];
  } ''
    # For acacia_sddf
    PYTHONPATH="${sources.sddf}:''${PYTHONPATH:-}" \
      python3 ${sources.srcRoot + "/crates/examples/microkit/http-server/meta.py"} \
        --board ${worldConfig.microkitConfig.board} \
        --elf-dir ${linkFarm "http-server-pds-unpatched" (lib.mapAttrsToList (_: pd: {
          name = "${pd.rootCrate.name}.elf";
          path = pd.elf;
        }) pds)} \
        --out-dir $out
  '';

  patchedPDs = runCommand "http-server-pds" {
    nativeBuildInputs = [
      buildPackages.llvmPackages_18.llvm
    ];
  } ''
    mkdir $out
    ${lib.concatMapStrings (pd: ''
      cp --no-preserve=mode ${pd.elf} $out/
    '') (lib.attrValues pds)}
    while read -r elf section blob; do
      llvm-objcopy --update-section .$section=${generated}/$blob $out/$elf
    done < ${generated}/patches.txt
  '';

in
lib.fix (self: callPlatform {
  system = microkit.mkSystem {
    searchPath = [
      patchedPDs
    ];
    systemXML = "${generated}/http-server.system";
  };
  extraPlatformArgs = lib.optionalAttrs canSimulate {
    extraQEMUArgs = [
      # MAC address must match meta.py
      "-device" "virtio-net-device,netdev=netdev0,mac=52:54:00:12:34:56"
      "-netdev" "user,id=netdev0,hostfwd=tcp::8080-:80,hostfwd=tcp::8443-:443"

      "-device" "virtio-blk-device,drive=blkdev0"
      "-blockdev" "node-name=blkdev0,read-only=on,driver=file,filename=${diskImage}/disk.img"
    ];
  };
} // {
  inherit pds generated patchedPDs;
  inherit largeDiskImage smallDiskImage diskImage;
} // lib.optionalAttrs canSimulate rec {
  automate =
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
        pkgs.requests
      ]);
    in
      writeScript "automate" ''
        #!${buildPackages.runtimeShell}
        set -eu
        ${py}/bin/python3 ${./automate.py} ${self.simulate}
      '';
})
