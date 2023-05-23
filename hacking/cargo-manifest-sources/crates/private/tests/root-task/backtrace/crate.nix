{ mk, localCrates, coreLicense, meAsAuthor }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "tests-root-task-backtrace";
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-backtrace
    sel4-backtrace-types
    sel4-backtrace-simple
    sel4-backtrace-embedded-debug-info
    sel4-root-task-runtime
  ];
  dependencies = {
    sel4-root-task-runtime.features = [ "alloc" "single-threaded" ];
    sel4-backtrace-simple.features = [ "alloc" ];
    sel4-backtrace.features = [ "full" ];
    sel4-backtrace-types.features = [ "full" ];
  };
}
