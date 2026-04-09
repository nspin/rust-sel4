{

  loaderTargetForTarget = target: {
    "aarch64" = "aarch64-unknown-none";
    "armv7" = "armv7a-none-eabi";
    "armv7a" = "armv7a-none-eabi";
    "riscv64gc" = "riscv64gc-unknown-none-elf";
    "riscv64imac" = "riscv64imac-unknown-none-elf";
    "riscv32imac" = "riscv32imac-unknown-none-elf";
    "riscv32imafc" = "riscv32imafc-unknown-none-elf";
  }.${firstSegment target};

  capdlInitializerTargetForTarget = target: {
    "aarch64" = "aarch64-sel4-roottask-minimal";
    "armv7" = "armv7a-sel4-roottask-minimal";
    "armv7a" = "armv7a-sel4-roottask-minimal";
    "riscv64gc" = "riscv64gc-sel4-roottask-minimal";
    "riscv64imac" = "riscv64imac-sel4-roottask-minimal";
    "riscv32imac" = "riscv32imac-sel4-roottask-minimal";
    "riscv32imafc" = "riscv32imafc-sel4-roottask-minimal";
    "x86_64" = "x86_64-sel4-roottask-minimal";
  }.${firstSegment target};


  mkRunner = body: ''
    set +x

    external_exe="$1"
    shift

    target_dir="$WORLD_TARGET_DIR"
    simulate_script="$WORLD_QEMU_SCRIPT"

    parent="$target_dir/runner"
    mkdir -p "$parent"
    d="$(mktemp -d --tmpdir="$parent")"

    echo 'd:' >&2
    echo "$d" >&2

    cleanup() {
      rm -rf "$d"
    }

    # trap cleanup EXIT

    exe_name="$(basename "$external_exe")"
    exe="$d/$exe_name"

    cp "$external_exe" "$exe"

    ${body}

    cargo run -p sel4-test-sentinels-wrapper -- "$simulate_script" "$image" "$@"

    stty echo
  '';

  rootTaskRunner = target: writeShellApplication {
    name = "root-task-runner";
    runtimeInputs = [
    ];
    checkPhase = "";
    text = mkRunner (if firstSegment target == "x86_64" then ''
      image="$exe"
    '' else ''
      image="$d/image.elf"

      cargo build \
        --config ${byTarget.${loaderTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-kernel-loader \
        --artifact-dir "$d"

      cargo run -p sel4-kernel-loader-add-payload -- \
        --loader "$d/sel4-kernel-loader" \
        --sel4-prefix "$SEL4_PREFIX" \
        --app "$exe" \
        -o "$image"
    '');
  };

  microkitRunner = writeShellApplication {
    name = "microkit-runner";
    runtimeInputs = [
      llvm
      (python312.withPackages (_: [
        sdfgen
      ]))
    ];
    text = mkRunner ''
      export PYTHONPATH="${toString ../../src/python}:''${PYTHONPATH:-}"

      (
        llvm-objcopy --dump-section .sdf_xml="$d/system.xml" "$exe" 2>/dev/null
      ) || (
        llvm-objcopy --dump-section .sdf_script="$d/system.py" "$exe"; \
        python3 "$d/system.py" \
            --board "$MICROKIT_BOARD" \
            -o "$d/system.xml"
      )

      image="$d/image.elf"

      "$MICROKIT_SDK/bin/microkit" "$d/system.xml" \
        --search-path "$d" \
        --board "$MICROKIT_BOARD" \
        --config "$MICROKIT_CONFIG" \
        -o "$image" \
        -r "$d/report.txt"
    '';
  };

  testfwRunner = target: writeShellApplication {
    name = "testfw-runner";
    excludeShellChecks = [
      "SC2317"
      "SC2329"
      "SC2154"
    ];
    runtimeInputs = [
      llvm
      capdl-tool
      (python312.withPackages (p: with p; [
        future six
        aenum sortedcontainers
        pyyaml pyelftools pyfdt
      ]))
    ];
    text = mkRunner (''
      export PYTHONPATH="${toString ../../src/python}:${sources.pythonCapDLTool}:''${PYTHONPATH:-}"

      llvm-objcopy --dump-section .capdl_script="$d/system.py" "$exe"

      script_out_dir="$d/cdl"

      python3 "$d/system.py" \
        --search-path "$d" \
        --object-sizes "$WORLD_OBJECT_SIZES" \
        -o "$script_out_dir"

      parse-capDL --object-sizes="$WORLD_OBJECT_SIZES" --json="$d/cdl.json" "$script_out_dir/spec.cdl"

      image="$d/image.elf"
      root_task="$d/root-task.elf"

      cargo build \
        --config ${byTarget.${capdlInitializerTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-capdl-initializer \
        --artifact-dir "$d"

      cargo run -p sel4-capdl-initializer-add-spec -- \
        -v \
        -e "$d/sel4-capdl-initializer.elf" \
        -f "$d/cdl.json" \
        -d "$script_out_dir/links" \
        --object-names-level 2 \
        --no-embed-frames \
        --no-deflate \
        -o "$root_task"
    '' + (if firstSegment target == "x86_64" then ''
      image="$root_task"
    '' else ''
      image="$d/image.elf"

      cargo build \
        --config ${byTarget.${loaderTargetForTarget target}} \
        --target-dir "$target_dir" \
        -p sel4-kernel-loader \
        --artifact-dir "$d"

      cargo run -p sel4-kernel-loader-add-payload -- \
        --loader "$d/sel4-kernel-loader" \
        --sel4-prefix "$SEL4_PREFIX" \
        --app "$root_task" \
        -o "$image"
    ''));
  };

}