{ mk, localCrates, versions, serdeWith, smoltcpWith }:

mk {
  package.name = "sel4-hal-adapters";
  package.authors = [
    "Nick Spinale <nick.spinale@coliasgroup.com>"
    "Ben Hamlin <hamlinb@galois.com>"
  ];
  dependencies = {
    inherit (versions) log;
    serde = serdeWith [];
    smoltcp = smoltcpWith [] // { optional = true; };
    sel4-bounce-buffer-allocator = { optional = true; };
    sel4-externally-shared = { features = [ "unstable" ]; optional = true; };
    sel4-shared-ring-buffer = { optional = true; };
    sel4-microkit = { default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-microkit
    sel4-microkit-message
    sel4-bounce-buffer-allocator
    sel4-externally-shared
    sel4-shared-ring-buffer
  ];
  features = {
    default = [
      "smoltcp-hal"
    ];
    smoltcp-hal = [
      "smoltcp"
      "sel4-bounce-buffer-allocator"
      "sel4-shared-ring-buffer"
      "sel4-externally-shared"
    ];
  };
}
