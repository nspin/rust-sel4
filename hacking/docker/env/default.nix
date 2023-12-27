let
  homeManagerPath =
    let
      rev = "d5824a76bc6bb93d1dce9ebbbcb09a9b6abcc224"; # branch release-23.11
    in builtins.fetchTarball {
      url = "https://github.com/nix-community/home-manager/archive/${rev}.tar.gz";
      sha256 = "sha256:16ab1k33aivqc5ighi95nh28pssbds5glz3bb371gb06qpiydihl";
    };

  pkgs = import <nixpkgs> {};

  # pkgs = (import ../../../hacking/nix {}).pkgs.build;

  # nixpkgsPath =
  #   let
  #     rev = "ad9ca03be8aaf8d6e458102e7d77370b7fe71ccf"; # branch release-23.11
  #   in builtins.fetchTarball {
  #     url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  #     sha256 = "sha256:05x1mcf9wkp024838mkaqr4y6kaf4mfcz1srsa2gcnv328bw0gya";
  #   };

  # pkgs = import nixpkgsPath {};

  home = import (homeManagerPath + "/modules") {
    inherit pkgs;
    configuration = ./config;
  };

in {
  inherit home;
  inherit (home) activationPackage;
  inherit (home.config.home) path;
}
