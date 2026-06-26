#!/usr/bin/env bash
# Bootstraps fonttools in a cached venv, then regenerates the CJK subset font.
# See build-cjk-subset.py for what it does and the SHS_TTC override.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
venv="${TMPDIR:-/tmp}/loom-fontsubset-venv"

if [ ! -x "$venv/bin/python" ]; then
  python3 -m venv "$venv"
  "$venv/bin/pip" install --quiet fonttools brotli
fi

exec "$venv/bin/python" "$here/build-cjk-subset.py" "$@"
