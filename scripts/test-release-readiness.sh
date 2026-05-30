#!/usr/bin/env bash
# Positive and negative checks for v2 release readiness gates.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-all}"
CLEANUP_PATHS=()

cleanup_release_readiness() {
  local path
  for path in "${CLEANUP_PATHS[@]}"; do
    rm -rf "$path"
  done
}
trap cleanup_release_readiness EXIT

tmp_copy() {
  local tmp
  tmp="$(mktemp -d)"
  mkdir -p "$tmp/.codex-plugin" "$tmp/rust/task-registry-flow-cli"
  cp "$ROOT/VERSION" "$tmp/VERSION"
  cp "$ROOT/plugin.json" "$tmp/plugin.json"
  cp "$ROOT/.codex-plugin/plugin.json" "$tmp/.codex-plugin/plugin.json"
  cp "$ROOT/MANIFEST.toml" "$tmp/MANIFEST.toml"
  cp "$ROOT/rust/task-registry-flow-cli/Cargo.toml" "$tmp/rust/task-registry-flow-cli/Cargo.toml"
  printf '%s\n' "$tmp"
}

check_version() {
  "$ROOT/scripts/release-version-check.sh" "$ROOT"

  local drift
  drift="$(tmp_copy)"
  printf '2.0.1\n' > "$drift/VERSION"
  if "$ROOT/scripts/release-version-check.sh" "$drift" >/tmp/release-version-drift.out 2>&1; then
    echo "version drift unexpectedly passed" >&2
    exit 1
  fi
  grep -q 'version mismatch' /tmp/release-version-drift.out

  local missing
  missing="$(tmp_copy)"
  rm "$missing/VERSION"
  if "$ROOT/scripts/release-version-check.sh" "$missing" >/tmp/release-version-missing.out 2>&1; then
    echo "missing VERSION unexpectedly passed" >&2
    exit 1
  fi
  grep -q 'missing VERSION' /tmp/release-version-missing.out
}

check_artifacts() {
  test -f "$ROOT/CHANGELOG.md"
  test -f "$ROOT/LICENSE"
  test -f "$ROOT/docs/releases/v2.md"
  test -f "$ROOT/rust/task-registry-flow-cli/deny.toml"
  grep -q '2.0.0' "$ROOT/CHANGELOG.md"
  grep -q 'Breaking changes' "$ROOT/CHANGELOG.md"
  grep -q 'Audit Policy' "$ROOT/docs/releases/v2.md"
  grep -q 'Unicode-3.0' "$ROOT/rust/task-registry-flow-cli/deny.toml"
  grep -q 'ignore = true' "$ROOT/rust/task-registry-flow-cli/deny.toml"
  grep -q 'CHANGELOG.md' "$ROOT/README.md"
  grep -q 'docs/releases/v2.md' "$ROOT/README.md"
}

check_status() {
  AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK=1 \
    AGENT_GOVERNANCE_ALLOW_ACTIVE_RELEASE_TASKS=1 \
    "$ROOT/scripts/status.sh" --release-source
  if "$ROOT/scripts/status.sh" --strict >/tmp/release-consumer-strict.out 2>&1; then
    echo "consumer strict status unexpectedly passed in plugin source checkout" >&2
    exit 1
  fi
  grep -q 'missing required CI artifact' /tmp/release-consumer-strict.out
}

check_audit() {
  AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 "$ROOT/scripts/release-audit.sh"
  local nested_root="$ROOT/.release-readiness-nested"
  CLEANUP_PATHS+=("$nested_root")
  mkdir -p "$nested_root/check"
  (
    cd "$nested_root/check"
    AGENT_GOVERNANCE_FORCE_MISSING_AUDIT_TOOLS=1 \
      AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 \
      "$ROOT/scripts/release-audit.sh" >/tmp/release-audit-nested.out 2>&1
  )
  grep -q 'source file limit ok' /tmp/release-audit-nested.out
  grep -q 'audit tool waiver active' /tmp/release-audit-nested.out

  local audit_copy
  audit_copy="$(mktemp -d)"
  CLEANUP_PATHS+=("$audit_copy")
  mkdir -p "$audit_copy/repo"
  tar -C "$ROOT" \
    --exclude='.git' \
    --exclude='target' \
    --exclude='rust/target' \
    --exclude='.release-readiness-nested' \
    -cf - . | tar -C "$audit_copy/repo" -xf -
  git init -q "$audit_copy/repo"
  mkdir -p "$audit_copy/repo/nested/check"
  python3 - "$audit_copy/repo/root-over-limit.md" <<'PY'
from pathlib import Path
import sys
Path(sys.argv[1]).write_text("root line\n" * 1601, encoding="utf-8")
PY
  if (
    cd "$audit_copy/repo/nested/check"
    AGENT_GOVERNANCE_FORCE_MISSING_AUDIT_TOOLS=1 \
      AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 \
      ../../scripts/release-audit.sh > /tmp/release-audit-over-limit.out 2>&1
  ); then
    echo "nested release audit unexpectedly missed root source-limit violation" >&2
    exit 1
  fi
  grep -q 'root-over-limit.md' /tmp/release-audit-over-limit.out

  if AGENT_GOVERNANCE_FORCE_MISSING_AUDIT_TOOLS=1 "$ROOT/scripts/release-audit.sh" >/tmp/release-audit-missing.out 2>&1; then
    echo "missing audit tools unexpectedly passed without waiver" >&2
    exit 1
  fi
  grep -q 'cargo-deny missing' /tmp/release-audit-missing.out
}

case "$MODE" in
  all)
    check_version
    check_artifacts
    check_status
    check_audit
    ;;
  version) check_version ;;
  artifacts) check_artifacts ;;
  status) check_status ;;
  audit) check_audit ;;
  *) echo "usage: test-release-readiness.sh [all|version|artifacts|status|audit]" >&2; exit 2 ;;
esac

echo "release readiness tests ok: $MODE"
