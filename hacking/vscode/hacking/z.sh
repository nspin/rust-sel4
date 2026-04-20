#!/usr/bin/env bash

set -eu -o pipefail

here="$(dirname $0)"

cargo run -- --manifest-path ../../../Cargo.toml \
    --exclude sel4-config-data \
    --exclude sel4-test-harness \
    --exclude tests-root-task-dafny-core \
    --exclude lionsos-sys \
    --exclude sddf-sys \
    --exclude sel4-newlib \
