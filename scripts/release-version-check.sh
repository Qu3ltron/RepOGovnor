#!/usr/bin/env bash
# Verify all release-version surfaces agree.
set -euo pipefail

if [[ $# -gt 1 ]]; then
  echo "usage: release-version-check.sh [root]" >&2
  exit 2
fi

ROOT="${1:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
ROOT="$(cd "$ROOT" && pwd)"
MANIFEST="${ROOT}/rust/task-registry-flow-cli/Cargo.toml"

if [[ ! -f "$MANIFEST" ]]; then
  echo "missing task-registry-flow Cargo.toml under ${ROOT}" >&2
  exit 1
fi

cd "$ROOT"
cargo run --locked --quiet --manifest-path "$MANIFEST" -- release-check version
