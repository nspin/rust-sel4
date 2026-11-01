#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ python312Packages
, sources
}:

with python312Packages;
buildPythonPackage {
  name = "acacia";
  pyproject = true;
  src = sources.acacia;

  pythonImportsCheck = [ "acacia" ];

  build-system = [ setuptools ];

  dependencies = [ libfdt pyelftools ];

  pythonRelaxDeps = true;
}
