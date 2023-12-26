#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

set -eu -o pipefail

# . ~/.nix-profile/etc/profile.d/nix.sh

path=$(nix-build nix -A path --no-out-link)
activationPackage=$(nix-build nix -A activationPackage --no-out-link)

nix-env -ir $path
$activationPackage/activate
