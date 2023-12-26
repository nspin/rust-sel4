{ pkgs, lib, config, ... }:

{
  imports = [
    ./vscode-server.nix
  ];

  home.stateVersion = "23.11";

  # home.username = "x";
  home.username = "root";

  home.homeDirectory =
    let
      inherit (config.home) username;
    in
      if username == "root" then "/root" else "/home/${username}";

  home.sessionPath = [
    "$HOME/.cargo/bin"
  ];

  # Is this a hack?
  targets.genericLinux.enable = true;

  # HACK
  manual.manpages.enable = false;

  programs.bash.enable = true;

  nix.enable = true;
  nix.package = pkgs.nix;
  nix.settings = {
    sandbox-fallback = false;

    keep-outputs = true;
    keep-derivations = true;

    experimental-features = [
      "nix-command"
      "flakes"
    ];

    extra-substituters = [
      "https://coliasgroup.cachix.org"
    ];

    extra-trusted-public-keys = [
      "coliasgroup.cachix.org-1:vYRVaHS5FCjsGmVVXlzF5LaIWjeEK17W+MHxK886zIE="
    ];
  };

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

    acl
    attr
    bzip2
    cpio
    diffutils
    gawk
    stdenv.cc.libc
    getent
    getconf
    gnugrep
    gnupatch
    gnused
    gnutar
    gzip
    xz
    less
    libcap
    ncurses
    netcat
    mkpasswd
    procps
    su
    time
    util-linux
    which
    zstd

    rustup
    strace
    gnumake
    nix-bash-completions
    gcc
    docker
  ];

  home.file = {
    ".inputrc".text = ''
      set editing-mode vi
      set show-mode-in-prompt on
    '';
  };
}
