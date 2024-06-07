{ mk, versions, localCrates }:

let
  x = mk {
    package.name = "banscii-xuartps-driver";
    dependencies = {
      inherit (versions) tock-registers;
      inherit (localCrates) banscii-uart-driver-traits;
    };
  };

in x // {
  package = builtins.removeAttrs x.package [ "authors" ];
}
