{ mk, versions, localCrates }:

mk {
  package.name = "sel4-xuartps-driver";
  package.authors = [
    "TODO"
  ];
  dependencies = {
    inherit (versions) tock-registers embedded-hal-nb;
    inherit (localCrates) sel4-driver-traits;
  };
}
