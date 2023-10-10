{ mk, localCrates, versions, virtioDriversWith }:

mk {
  package.name = "microkit-http-server-example-smoltcp-phy-impl";
  package.authors = [
    "Nick Spinale <nick.spinale@coliasgroup.com>"
    "Ben Hamlin <hamlinb@galois.com>"
  ];
  dependencies = {
    inherit (versions) log;
    virtio-drivers = virtioDriversWith [ "alloc" ];
    smoltcp = {
      version = versions.smoltcp;
      default-features = false;
        features = [
          "proto-ipv4" "medium-ethernet" "socket-raw"
      ];
    };
    sel4-externally-shared = { features = [ "unstable" ]; };
    sel4-microkit = { default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4
    sel4-bounce-buffer-allocator
    sel4-externally-shared
    sel4-immediate-sync-once-cell
    sel4-hal-adapters
    sel4-logging
    sel4-microkit
    sel4-microkit-message
    sel4-shared-ring-buffer
    sel4-sync
    microkit-http-server-example-virtio-hal-impl
  ];
}
