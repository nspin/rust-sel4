{ pkgs ? import <nixpkgs> { }
, lib ? pkgs.lib
}:

let

  users = {

    root = {
      uid = 0;
      gid = globalGroups.root.gid;
      home = "/root";
      groups = [ "root" ];
    };

    nobody = {
      uid = 65534;
      groups = [ "nobody" ];
    };

  } // lib.listToAttrs (lib.forEach (lib.lists.range 1 32) (n:
    lib.nameValuePair "nixbld${toString n}" {
      uid = 30000 + n;
      gid = 30000;
      groups = [ "nixbld" ];
      description = "Nix build user ${toString n}";
    }
  ));

  groups = {
    root.gid = 0;
    nixbld.gid = 30000;
    nobody.gid = 65534;
  };

  groupMembers = lib.flip lib.mapAttrs groups (group: _:
    lib.flip lib.filter (lib.attrNames users) (user: lib.elem group (users.${user}.groups or []))
  );

  globalGroups = groups;

  formatPasswdEntry =
    k:
    { uid
    , gid ? globalGroups.nobody.gid
    , description ? ""
    , home ? "/var/empty"
    , shell ? "/bin/false"
    , groups ? []
    }:
    "${k}:x:${toString uid}:${toString gid}:${description}:${home}:${shell}\n";

  formatShadowEntry = k: { ... }:
    "${k}:!:1::::::\n";

  formatGroupEntry = k: members:
    "${k}:x:${toString groups.${k}.gid}:${lib.concatStringsSep "," members}\n";

  passwdFile = pkgs.writeText "passwd" (
    lib.concatStrings
      (lib.attrValues (lib.mapAttrs formatPasswdEntry users))
  );

  shadowFile = pkgs.writeText "shadow" (
    lib.concatStrings
      (lib.attrValues (lib.mapAttrs formatShadowEntry users))
  );

  groupFile = pkgs.writeText "group" (
    lib.concatStrings
      (lib.attrValues (lib.mapAttrs formatGroupEntry groupMembers))
  );

  initialEnv = pkgs.buildEnv {
    name = "env";
    paths = with pkgs; [
      bashInteractive
      coreutils
      nix
      cacert
    ];
  };

  setup = pkgs.writeScript "setup" ''
    #!${pkgs.bashInteractive}/bin/bash

    set -eu -o pipefail

    registration=/nix-support/registration

    ${pkgs.nix}/bin/nix-store --init
    ${pkgs.nix}/bin/nix-store --load-db < $registration
    ${pkgs.coreutils}/bin/rm $registration
    ${pkgs.nix}/bin/nix-env -i ${initialEnv}
  '';

  baseSystem = pkgs.runCommand "base-system" {} ''
    mkdir -p \
      $out/etc \
      $out/bin \
      $out/usr/bin \
      $out/tmp \
      $out/var/tmp \
      $out/root \
      $out/nix-support

    ln -s ${passwdFile} $out/etc/passwd
    ln -s ${groupFile} $out/etc/group
    ln -s ${shadowFile} $out/etc/shadow

    ln -s ${pkgs.bashInteractive}/bin/bash $out/bin/sh
    ln -s ${pkgs.coreutils}/bin/env $out/usr/bin

    ln -s ${setup} $out/nix-support/setup
  '';

  baseSystemClosureInfo = pkgs.closureInfo { rootPaths = [ baseSystem ]; };

  system = pkgs.runCommand "system" {} ''
    root=$out/root

    mkdir -p $root

    cp -r ${baseSystem}/* $root
    chmod +w $root/nix-support
    cp ${baseSystemClosureInfo}/registration $root/nix-support

    for storePath in $(cat ${baseSystemClosureInfo}/store-paths); do
      cp -r --parents $storePath $root
    done
  '';

in
system

# notes:
# chmod 1777 /tmp /var/tmp
# "USER=root"
