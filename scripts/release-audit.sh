#!/usr/bin/env bash
# Release audit gate for the plugin source package.
set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: release-audit.sh" >&2
  exit 2
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
ROOT="$(cd "$ROOT" && pwd)"
MANIFEST="${ROOT}/rust/task-registry-flow-cli/Cargo.toml"
PACKAGE_DIR="${ROOT}/rust/task-registry-flow-cli"
cd "$ROOT"

cargo run --locked --quiet --manifest-path "$MANIFEST" -- source-limit check
cargo run --locked --quiet --manifest-path "$MANIFEST" -- release-check all
cargo fmt --manifest-path "$MANIFEST" -- --check
cargo test --locked --manifest-path "$MANIFEST"
cargo clippy --locked --manifest-path "$MANIFEST" -- -D warnings

dupes="$(cargo tree --locked --manifest-path "$MANIFEST" -d 2>&1 || true)"
if [[ "$dupes" != *"nothing to print"* ]]; then
  printf '%s\n' "$dupes" >&2
  echo "duplicate dependency check failed" >&2
  exit 1
fi

missing=0
if [[ "${AGENT_GOVERNANCE_FORCE_MISSING_AUDIT_TOOLS:-0}" == "1" ]] || ! command -v cargo-audit >/dev/null 2>&1; then
  echo "cargo-audit missing" >&2
  missing=1
fi
if [[ "${AGENT_GOVERNANCE_FORCE_MISSING_AUDIT_TOOLS:-0}" == "1" ]] || ! command -v cargo-deny >/dev/null 2>&1; then
  echo "cargo-deny missing" >&2
  missing=1
fi
if [[ "$missing" -eq 1 ]]; then
  if [[ "${AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER:-0}" == "1" ]]; then
    if [[ "${AGENT_GOVERNANCE_FINAL_RELEASE:-0}" == "1" ]]; then
      echo "audit tool waiver forbidden in final release mode" >&2
      exit 1
    fi
    if [[ -z "${AGENT_GOVERNANCE_AUDIT_TOOL_WAIVER_REASON:-}" ]]; then
      echo "audit tool waiver requires AGENT_GOVERNANCE_AUDIT_TOOL_WAIVER_REASON" >&2
      exit 1
    fi
    echo "audit tool waiver active: ${AGENT_GOVERNANCE_AUDIT_TOOL_WAIVER_REASON}" >&2
    exit 0
  fi
  echo "install cargo-audit and cargo-deny, or set AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 for a governed waiver" >&2
  exit 1
fi

(
  cd "$PACKAGE_DIR"
  cargo audit --file Cargo.lock
  cargo deny --locked check --config deny.toml
)

echo "release audit ok"
