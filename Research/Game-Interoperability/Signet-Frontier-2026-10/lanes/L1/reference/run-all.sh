#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
vectors="$here/../schema/test-vectors"

python3 "$here/python/reference.py" "$vectors"
node "$here/javascript/reference.mjs" "$vectors"
