let
  configuration = { pkgs, lib, config, ... }:
    let
    in {
      home.stateVersion = "23.11";

      # home.username = "x";
      home.username = "root";

      home.homeDirectory =
        let
          inherit (config.home) username;
        in
          if username == "root" then "/root" else "/home/${username}";

      manual.manpages.enable = false;

      programs.bash.enable = true;

      home.packages = with pkgs; [
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
      ];

      home.file = {
        ".inputrc".text = ''
          set editing-mode vi
          set show-mode-in-prompt on
        '';
      };
    };

in
let
  homeManagerPath =
    let
      rev = "d5824a76bc6bb93d1dce9ebbbcb09a9b6abcc224"; # branch release-23.11
    in builtins.fetchTarball {
      url = "https://github.com/nix-community/home-manager/archive/${rev}.tar.gz";
      sha256 = "sha256:16ab1k33aivqc5ighi95nh28pssbds5glz3bb371gb06qpiydihl";
    };

  pkgs = (import ../../hacking/nix {}).pkgs.build;

  home = import (homeManagerPath + "/modules") {
    inherit configuration pkgs;
  };

in {
  inherit home;
}
