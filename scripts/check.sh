#!/usr/bin/env bash
# Item gate: fast checks to run after every item, before its commit.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== fmt =="
cargo fmt --all -- --check

echo "== clippy =="
cargo clippy --all-targets -- -D warnings

echo "== build =="
cargo build --all-targets

echo "check.sh: OK"
