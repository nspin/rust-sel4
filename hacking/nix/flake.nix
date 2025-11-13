{
  inputs = {
    nixpkgs.url = "github:coliasgroup/nixpkgs?ref=0a44fc400a2ea73cee67c4effbae10b6bb254da8";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        legacyPackages = import ./. {
          nixpkgsFnArgs = {
            localSystem.system = system;
          };
        };
      }
    );
}
