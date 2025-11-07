{ lib
, hostPlatform
, writers
, writeShellApplication
, pkgsBuildBuild

, otherSplices
, selfTopLevel
, sel4-kernel-loader-add-payload

, worldConfig
, seL4ForBoot
, seL4RustEnvVars
, sel4-kernel-loader
}:

let
  inherit (selfTopLevel.cargoConfig) utils;

  hostCPUName = hostPlatform.parsed.cpu.name;
  hostRustTargetRiscVArch = hostPlatform.this.rustTargetRiscVArch or null;
  hostIsMicrokit = worldConfig.isMicrokit;

  isRelevant = target:
    let
      seg = utils.firstSegment target;
      cpuName = {
        "armv7" = "arm";
        "armv7a" = "arm";
        "riscv64gc" = "riscv64";
        "riscv64imac" = "riscv64";
        "riscv32imac" = "riscv32";
        "riscv32imafc" = "riscv32";
      }.${seg} or seg;
      rustTargetRiscVArch = hostPlatform.this.rustTargetRiscVArch or null;
      isMicrokit = utils.hasSegment "microkit" target;
    in
      lib.all lib.id [
        (cpuName == hostCPUName)
        (rustTargetRiscVArch == hostRustTargetRiscVArch)
        (isMicrokit == hostIsMicrokit)
      ]
    ;

  hostRunner = pkgsBuildBuild.writeShellApplication {
    name = "runner";

    runtimeInputs = [
      pkgsBuildBuild.qemu
      otherSplices.selfBuildBuild.sel4-kernel-loader-add-payload
    ];

    text = ''
      f=/home/x/i/rust-sel4/tmp/foo.elf

      sel4-kernel-loader-add-payload \
        --loader ${sel4-kernel-loader.elf} \
        --sel4-prefix ${seL4ForBoot} \
        --app "$1" \
        -o $f

      qemu-system-aarch64 \
        -machine virt,virtualization=on \
        -cpu cortex-a57 \
        -smp 2 \
        -m size=2048M \
        -nographic \
        -serial mon:stdio \
        -kernel $f
    '';
  };

  configForTarget = isBuiltin: target:
    let
    in
      lib.optionalAttrs (isRelevant target) {
        target.${target} = {
          runner = "${hostRunner}/bin/runner";
        };
      }
    ;

in
writers.writeTOML "config-world.toml" (utils.clobberAttrs ([
  {
    env = seL4RustEnvVars;
  }
] ++ map (configForTarget true) utils.builtinBareMetalTargets
  ++ map (configForTarget false) utils.seL4Targets
))
