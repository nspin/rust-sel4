{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.requirements = [ "sel4-config" ];
  package.name = "sel4-config-macros";
  lib.proc-macro = true;
  nix.local.dependencies = with localCrates; [
    sel4-config-generic-macro-impls
    sel4-config-data
  ];
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
}
