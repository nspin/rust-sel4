#!/usr/bin/env bash

set -eu -o pipefail

here="$(dirname $0)"

# comm -23 \
#   <(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sort -u) \
#   <(cargo tree --workspace \
#     --invert sel4-config-data \
#     --invert sel4-test-harness \
#     --invert sel4-reset \
#     --invert tests-root-task-dafny-core \
#     --prefix none --format '{p}' | cut -d ' ' -f 1 | sort -u) \
#   > $here/excludes.txt

cargo tree --workspace \
    --invert sel4-config-data \
    --invert sel4-test-harness \
    --invert sel4-reset \
    --invert tests-root-task-dafny-core \
    --prefix none --format '{p}' \
| cut -d ' ' -f 1 \
| sort -u \
> $here/excludes.txt
