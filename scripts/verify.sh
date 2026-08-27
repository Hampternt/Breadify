#!/usr/bin/env bash
# Pack gate: everything check.sh does, plus the full test suite.
# Nothing merges without this.
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/check.sh

echo "== tests =="
cargo test --all-targets

echo "verify.sh: OK"
