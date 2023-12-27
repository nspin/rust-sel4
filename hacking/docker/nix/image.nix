{ pkgs ? import <nixpkgs> { }
, lib ? pkgs.lib
, name ? "mininix"
, tag ? "latest"
, extraPkgs ? []
, maxLayers ? 100
}:
let
  defaultPkgs = with pkgs; [
    nix
    bashInteractive
    coreutils-full
    gnutar
    gzip
    gnugrep
    which
    curl
    less
    wget
    man
    cacert.out
    findutils
    iana-etc
    git
    openssh
  ] ++ extraPkgs;

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
    }: "${k}:x:${toString uid}:${toString gid}:${description}:${home}:${shell}"
  );
  passwdContents = (
    lib.concatStringsSep "\n"
      (lib.attrValues (lib.mapAttrs userToPasswd users))
  );

  userToShadow = k: { ... }: "${k}:!:1::::::";
  shadowContents = (
    lib.concatStringsSep "\n"
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
    "${k}:x:${toString gid}:${lib.concatStringsSep "," members}";
  groupContents = (
    lib.concatStringsSep "\n"
      (lib.attrValues (lib.mapAttrs groupToGroup groups))
  );

  # TODO SSL_CERT_FILE=$NIX_SSL_CERT_FILE

  setup = with pkgs; writeShellApplication {
    name = "setup";
    text = ''
      nix-env -i ${initialEnv}
    '';
  };

  initialEnv = pkgs.buildEnv {
    name = "env";
    paths = defaultPkgs;
  };

  baseSystem =
    let
    in
    pkgs.runCommand "base-system" {
      inherit passwdContents groupContents shadowContents;
      passAsFile = [
        "passwdContents"
        "groupContents"
        "shadowContents"
      ];
      allowSubstitutes = false;
      preferLocalBuild = true;
    } ''
      env
      set -x
      mkdir -p $out/etc

      cat $passwdContentsPath > $out/etc/passwd
      echo "" >> $out/etc/passwd

      cat $groupContentsPath > $out/etc/group
      echo "" >> $out/etc/group

      cat $shadowContentsPath > $out/etc/shadow
      echo "" >> $out/etc/shadow

      mkdir $out/tmp

      mkdir -p $out/var/tmp

      mkdir -p $out/root

      mkdir -p $out/bin $out/usr/bin
      ln -s ${pkgs.coreutils}/bin/env $out/usr/bin/env
      ln -s ${pkgs.bashInteractive}/bin/bash $out/bin/sh

      ln -s ${setup} $out/setup
      ln -s ${pkgs.coreutils}/bin/sleep $out/sleep
    '';

in
pkgs.dockerTools.buildLayeredImageWithNixDb {

  inherit name tag maxLayers;

  contents = [ baseSystem ];

  # extraCommands = ''
  #   rm -rf nix-support
  #   ln -s /nix/var/nix/profiles nix/var/nix/gcroots/profiles
  # '';
  fakeRootCommands = ''
    chmod 1777 tmp
    chmod 1777 var/tmp
  '';

  config = {
    Cmd = [ "/bin/sh" ];
    Env = [
      "USER=root"
      # "PATH=${lib.concatStringsSep ":" [
      #   "/root/.nix-profile/bin"
      #   "/nix/var/nix/profiles/default/bin"
      #   "/nix/var/nix/profiles/default/sbin"
      # ]}"
      # "MANPATH=${lib.concatStringsSep ":" [
      #   "/root/.nix-profile/share/man"
      #   "/nix/var/nix/profiles/default/share/man"
      # ]}"
      # "SSL_CERT_FILE=/nix/var/nix/profiles/default/etc/ssl/certs/ca-bundle.crt"
      # "GIT_SSL_CAINFO=/nix/var/nix/profiles/default/etc/ssl/certs/ca-bundle.crt"
      # "NIX_SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
    ];
  };

}
