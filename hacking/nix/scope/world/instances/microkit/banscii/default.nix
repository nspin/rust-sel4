#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ lib, stdenv
, buildPackages, pkgsBuildBuild
, linkFarm, symlinkJoin, writeText, writeScript, runCommand
, python3Packages
, microkit
, acacia
, mkTask
, sources
, crates
, crateUtils
, mkSeL4RustTargetTriple
, worldConfig
, seL4ConfigJSON
, callPlatform

, canSimulate
, mkPD
}:

let
  inherit (worldConfig.microkitConfig) board;

  # Acacia reads the layouts of configuration structures from these protection domains' DWARF
  mkPDWithConfigs = args: mkPD (args // {
    extraProfile = crateUtils.clobber [
      (args.extraProfile or {})
      {
        strip = false;
        package.${args.rootCrate.name}.debug = true;
      }
    ];
  });

  pdsWithConfigs = {
    serial-driver = mkPDWithConfigs rec {
      rootCrate = crates.banscii-serial-driver;
      features = [ "board-${board}" ];
    };
    assistant = mkPDWithConfigs rec {
      rootCrate = crates.banscii-assistant;
      release = true;
      targetTriple = mkSeL4RustTargetTriple { minimal = false; };
    };
  };

  pds = pdsWithConfigs // {
    artist = mkPD rec {
      rootCrate = crates.banscii-artist;
      release = true;
      extraProfile = {
        # For RSA key generation
        build-override = {
          opt-level = 2;
        };
      };
    };
  };

  srcPath = relativePath: sources.srcRoot + "/crates/examples/microkit/banscii/${relativePath}";

  generated = runCommand "banscii-system" {
    nativeBuildInputs = [
      acacia
    ];
  } ''
    # For acacia_sddf
    PYTHONPATH="${sources.sddf}:''${PYTHONPATH:-}" \
      python3 ${srcPath "meta.py"} \
        --board ${board} \
        --elf-dir ${linkFarm "banscii-pds-unpatched" (lib.mapAttrsToList (_: pd: {
          name = "${pd.rootCrate.name}.elf";
          path = pd.elf;
        }) pdsWithConfigs)} \
        --out-dir $out
  '';

  patchedPDs = runCommand "banscii-pds" {
    nativeBuildInputs = [
      buildPackages.llvmPackages_18.llvm
    ];
  } ''
    mkdir $out
    ${lib.concatMapStrings (pd: ''
      cp --no-preserve=mode ${pd.elf} $out/
    '') (lib.attrValues pdsWithConfigs)}
    while read -r elf section blob; do
      llvm-objcopy --update-section .$section=${generated}/$blob $out/$elf
    done < ${generated}/patches.txt
  '';

in
lib.fix (self: callPlatform {
  system = microkit.mkSystem {
    searchPath = [
      patchedPDs
      "${pds.artist}/bin"
    ];
    systemXML = "${generated}/banscii.system";
  };
} // {
  inherit pds generated patchedPDs;
} // lib.optionalAttrs canSimulate rec {
  automate =
    let
      py = buildPackages.python3.withPackages (pkgs: [
        pkgs.pexpect
      ]);
    in
      writeScript "automate" ''
        #!${buildPackages.runtimeShell}
        set -eu
        ${py}/bin/python3 ${./automate.py} ${self.simulate}
      '';
})
