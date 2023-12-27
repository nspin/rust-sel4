{ pkgs ? import <nixpkgs> { }
, lib ? pkgs.lib
}:

let

  defaultPkgs = with pkgs; [
    bashInteractive
    coreutils
    nix
    cacert
    iana-etc
  ];

  users = {

    root = {
      uid = 0;
      shell = "${pkgs.bashInteractive}/bin/bash";
      home = "/root";
      gid = 0;
      groups = [ "root" ];
      description = "System administrator";
    };

    nobody = {
      uid = 65534;
      shell = "${pkgs.shadow}/bin/nologin";
      home = "/var/empty";
      gid = 65534;
      groups = [ "nobody" ];
      description = "Unprivileged account (don't use!)";
    };

  } // lib.listToAttrs (
    map
      (
        n: {
          name = "nixbld${toString n}";
          value = {
            uid = 30000 + n;
            gid = 30000;
            groups = [ "nixbld" ];
            description = "Nix build user ${toString n}";
          };
        }
      )
      (lib.lists.range 1 32)
  );

  groups = {
    root.gid = 0;
    nixbld.gid = 30000;
    nobody.gid = 65534;
  };

  userToPasswd = (
    k:
    { uid
    , gid ? 65534
    , home ? "/var/empty"
    , description ? ""
    , shell ? "/bin/false"
    , groups ? [ ]
    }: "${k}:x:${toString uid}:${toString gid}:${description}:${home}:${shell}\n"
  );
  passwdFile = pkgs.writeText "passwd" (
    lib.concatStrings
      (lib.attrValues (lib.mapAttrs userToPasswd users))
  );

  userToShadow = k: { ... }: "${k}:!:1::::::\n";
  shadowFile = pkgs.writeText "shadow" (
    lib.concatStrings
      (lib.attrValues (lib.mapAttrs userToShadow users))
  );

  # Map groups to members
  # {
  #   group = [ "user1" "user2" ];
  # }
  groupMemberMap = (
    let
      # Create a flat list of user/group mappings
      mappings = (
        builtins.foldl'
          (
            acc: user:
              let
                groups = users.${user}.groups or [ ];
              in
              acc ++ map
                (group: {
                  inherit user group;
                })
                groups
          )
          [ ]
          (lib.attrNames users)
      );
    in
    (
      builtins.foldl'
        (
          acc: v: acc // {
            ${v.group} = acc.${v.group} or [ ] ++ [ v.user ];
          }
        )
        { }
        mappings)
  );

  groupToGroup = k: { gid }:
    let
      members = groupMemberMap.${k} or [ ];
    in
    "${k}:x:${toString gid}:${lib.concatStringsSep "," members}\n";
  groupFile = pkgs.writeText "group" (
    lib.concatStrings
      (lib.attrValues (lib.mapAttrs groupToGroup groups))
  );

  initialEnv = pkgs.buildEnv {
    name = "env";
    paths = defaultPkgs;
  };

  setup = pkgs.writeScript "setup" ''
    #!${pkgs.bashInteractive}/bin/bash

    set -eu -o pipefail

    ${pkgs.nix}/bin/nix-store --init
    ${pkgs.nix}/bin/nix-store --load-db < /nix-support/registration
    ${pkgs.coreutils}/bin/rm /nix-support/registration
    ${pkgs.nix}/bin/nix-env -i ${initialEnv}
  '';

  baseSystem = pkgs.runCommand "base-system" {} ''
    mkdir -p $out/etc
    mkdir -p $out/bin
    mkdir -p $out/usr/bin
    mkdir -p $out/tmp
    mkdir -p $out/var/tmp
    mkdir -p $out/root
    mkdir -p $out/nix-support

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
