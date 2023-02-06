{ lib, buildPackages
, buildCrateInLayersHere, buildSysroot, crateUtils
, crates, injectPhdrs
, defaultRustTargetInfo
, seL4ForUserspace
} @ scopeArgs:

{ commonModifications ? {}
, lastLayerModifications ? {}

, rootTaskStackSize ? 4096 * 8
, rootTaskHeapSize ? 4096 * 16

, extraProfile ? {}
, replaceSysroot ? null
, injectPhdrs ? false

, rustTargetInfo ? defaultRustTargetInfo
, release ? true
, ...
} @ args:

let
  profile = if release then "release" else "dev";

  profiles = crateUtils.clobber [
    {
      profile.release = {
        lto = true;
      };
    }
    {
      profile.${profile} = {
        codegen-units = 1;
        incremental = false;
      } // extraProfile;
    }
  ];

  sysroot = (if replaceSysroot != null then replaceSysroot else buildSysroot) {
    inherit release rustTargetInfo;
    extraManifest = profiles;
  };

  maybeInjectPhdrs = if injectPhdrs then scopeArgs.injectPhdrs else lib.id;

  theseCommonModifications = crateUtils.elaborateModifications {
    modifyManifest = lib.flip lib.recursiveUpdate profiles;
    modifyConfig = lib.flip lib.recursiveUpdate {
      target.${rustTargetInfo.name}.rustflags = [
        "--sysroot" sysroot
      ];
    };
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      # TODO
      # hardeningDisable = [ "all" ];

      LIBCLANG_PATH = "${lib.getLib buildPackages.llvmPackages.libclang}/lib";

      dontStrip = true;
      dontPatchELF = true;
    });
  };

  theseLastLayerModifications = crateUtils.elaborateModifications {
    modifyDerivation = drv: drv.overrideAttrs (self: super: {
      SEL4_PREFIX = seL4ForUserspace;
      SEL4_RUNTIME_STACK_SIZE = rootTaskStackSize;
      SEL4_RUNTIME_HEAP_SIZE = rootTaskHeapSize;

      passthru = (super.passthru or {}) // {
        elf = maybeInjectPhdrs "${self.finalPackage}/bin/${args.rootCrate.name}.elf";
      };
    });
  };

  prunedArgs = builtins.removeAttrs args [
    "extraProfile"
    "replaceSysroot"
    "injectPhdrs"
    "rootTaskStackSize"
    "rootTaskHeapSize"
  ];

in

buildCrateInLayersHere (prunedArgs // {

  commonModifications = crateUtils.composeModifications
    (crateUtils.elaborateModifications commonModifications)
    theseCommonModifications
  ;

  lastLayerModifications = crateUtils.composeModifications
    (crateUtils.elaborateModifications lastLayerModifications)
    theseLastLayerModifications
  ;
})
