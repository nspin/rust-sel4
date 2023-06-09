{ lib, stdenv, hostPlatform, buildPackages
, writeScript, linkFarm
, overrideCC, libcCross
, which, strace
, llvmPackages

, crates
, mkTask
, seL4RustTargetInfoWithConfig

, mkInstance
}:

let
  stdenvWithLibc =
    let
      bintools = stdenv.cc.bintools.override {
        libc = libcCross;
        noLibc = false;
      };
    in
      stdenv.override {
        cc = stdenv.cc.override {
          libc = libcCross;
          noLibc = false;
          inherit bintools;
        };
      };

  stdenvWithLld = overrideCC stdenvWithLibc (stdenvWithLibc.cc.override {
    bintools = buildPackages.llvmPackages.bintools;
  });

  ccWrapper = writeScript "this-cc-wrapper" ''
    #!${buildPackages.runtimeShell}
    # env
    # which ${stdenvWithLld.cc.targetPrefix}cc
    # exit 1
    exec strace -f -e trace=file ${stdenvWithLld.cc.targetPrefix}cc $@
  '';

  rustTargetInfo = seL4RustTargetInfoWithConfig {
    # minimal = true;
    minimal = false;
  };

  instance = mkInstance {
    rootTask = mkTask rec {

      # stdenv = stdenvWithLld;
      stdenv = stdenvWithLibc;

      rootCrate = crates.tests-root-task-c;

      release = false;

      inherit rustTargetInfo;

      lastLayerModifications = {
        modifyDerivation = drv: drv.overrideAttrs (self: super: {
          # NIX_DEBUG = 3;
          nativeBuildInputs = super.nativeBuildInputs ++ [
            which
            strace
          ];

          NIX_DEBUG = 1;
        });
        modifyConfig = old: lib.recursiveUpdate old {
          target.${rustTargetInfo.name} = {

            # linker = "${stdenv.cc.targetPrefix}ld.lld";
            # rustflags = (old.target.${rustTargetInfo.name}.rustflags or []) ++ [
            #   "-C" "linker-flavor=ld"
            #   "-C" "link-arg=-lc"
            # ];

            # NOTE
            # This should work, but it doesn't.
            # TODO
            # Investigate
            linker = "${stdenv.cc.targetPrefix}cc";
            # linker = ccWrapper;
            rustflags = (old.target.${rustTargetInfo.name}.rustflags or []) ++ [
              "-C" "linker-flavor=gcc"
              # "-C" "linker-flavor=gcc-lld"
              "-C" "link-arg=-nostartfiles"
              "-C" "default-linker-libraries=on"
              "-Z" "gcc-ld=lld"
              # "-C" "link-arg=-fuse-ld=lld"
            ];
          };
        };
      };
    };

    isSupported = false;
    canAutomate = true;

  };

in {
  inherit instance;
}
