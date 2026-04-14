#!/usr/bin/env bash

set -eu -o pipefail

here="$(dirname $0)"

excludes_out=$here/../excludes.txt

comm -23 \
  <(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sort -u) \
  <(cargo tree "$@" --prefix none --format '{p}' \
    | grep '(/' \
    | cut -d ' ' -f 1 \
    | sort -u \
  ) \
  > $excludes_out


# hacking/vscode/gen-excludes-hacking/x.sh --config .cargo/gen/target/aarch64-sel4-microkit.toml --config .cargo/gen/world/aarch64.microkitDefault.toml -p microkit-http-server-example-server
