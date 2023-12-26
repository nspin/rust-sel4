#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

set -eu -o pipefail

. ~/.nix-profile/etc/profile.d/nix.sh

activationPackage=$(nix-build -A home.activationPackage --no-out-link)
path=$(nix-build -A home.config.home.path --no-out-link)

nix-env -ir $path

$activationPackage/activate
