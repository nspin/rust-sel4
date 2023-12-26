{ pkgs, lib, ... }:

with lib;

let
  nixosVSCodeServerPath =
    let
      rev = "1e1358493df6529d4c7bc4cc3066f76fd16d4ae6";
    in builtins.fetchTarball {
      url = "https://github.com/nix-community/nixos-vscode-server/archive/${rev}.tar.gz";
      sha256 = "sha256:0sz8njfxn5bw89n6xhlzsbxkafb6qmnszj4qxy2w0hw2mgmjp829";
    };

  auto-fix-vscode-server = pkgs.callPackage (nixosVSCodeServerPath + "/pkgs/auto-fix-vscode-server.nix") {};

in {
  options = {
    x = mkOption {
      type = types.package;
    };
  };
  config = {
    x = auto-fix-vscode-server;
    home.packages = [
      auto-fix-vscode-server
    ];
  };
}
