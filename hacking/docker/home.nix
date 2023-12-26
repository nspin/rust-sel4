{ pkgs, lib, config, ... }:

{
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
}
