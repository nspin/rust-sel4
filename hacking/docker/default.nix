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
    inherit pkgs;
    configuration = ./home.nix;
  };

in {
  inherit home;
  inherit (home) activationPackage;
  inherit (home.config.home) path;
}
