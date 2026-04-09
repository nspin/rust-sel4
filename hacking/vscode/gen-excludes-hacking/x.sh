#!/usr/bin/env bash

set -eu -o pipefail

here="$(dirname $0)"

excludes_out=$here/../excludes.txt

cargo tree "$@" --prefix none --format '{p}' \
  | grep '(/' \
  | cut -d ' ' -f 1 \
  | sort -u \
  > $excludes_out
