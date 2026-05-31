#!/usr/bin/env bash
# Smoke test driver for the Agent Governance plugin.
# Builds the Rust CLI and runs every read-only subcommand + supporting scripts.
#
# Usage: bash .claude/skills/run-governance-plugin/smoke.sh [--quick]
#   --quick  Use debug build (faster compile, no optimisation).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

MODE="${1:-full}"
CARGO_MANIFEST="$REPO_ROOT/rust/task-registry-flow-cli/Cargo.toml"
CARGO_TARGET_DIR="${AGENT_GOVERNANCE_CARGO_TARGET_DIR:-/tmp/agent-governance-cargo-target}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass=0
fail=0
warn=0

die() { echo -e "${RED}fatal: $*${NC}" >&2; exit 1; }

# ── helpers ────────────────────────────────────────────

check() {
  local label="$1"; shift
  printf "  %-50s " "$label ..."
  local out ec=0
  out=$("$@" 2>&1) || ec=$?
  if [ "$ec" -eq 0 ]; then
    echo -e "${GREEN}OK${NC}"
    pass=$((pass + 1))
  else
    echo -e "${RED}FAIL (exit=$ec)${NC}"
    echo "$out" | sed 's/^/    /' | head -5
    fail=$((fail + 1))
  fi
}

check_output() {
  local label="$1" expected="$2"; shift 2
  printf "  %-50s " "$label ..."
  local actual ec=0
  actual=$("$@" 2>&1) || ec=$?
  if echo "$actual" | grep -qF "$expected"; then
    echo -e "${GREEN}OK${NC}"
    pass=$((pass + 1))
  else
    echo -e "${RED}FAIL (exit=$ec)${NC}  expected substring: '$expected'"
    echo "$actual" | sed 's/^/    /' | head -5
    fail=$((fail + 1))
  fi
}

# Pass if the command exits 0 OR prints the expected substring (allows known
# non-zero exits, e.g. status.sh failing on untracked governance files).
check_any() {
  local label="$1" expected="$2"; shift 2
  printf "  %-50s " "$label ..."
  local actual ec=0
  actual=$("$@" 2>&1) || ec=$?
  if [ "$ec" -eq 0 ] || echo "$actual" | grep -qF "$expected"; then
    if [ "$ec" -ne 0 ]; then
      echo -e "${YELLOW}WARN (exit=$ec, expected)${NC}"
      warn=$((warn + 1))
    else
      echo -e "${GREEN}OK${NC}"
      pass=$((pass + 1))
    fi
  else
    echo -e "${RED}FAIL (exit=$ec)${NC}"
    echo "$actual" | sed 's/^/    /' | head -5
    fail=$((fail + 1))
  fi
}

# ── main ───────────────────────────────────────────────

echo ""
echo "=== Agent Governance Plugin Smoke Test ==="
echo ""

echo "--- Build ---"
if [ "$MODE" = "--quick" ]; then
  check "cargo build (debug)" cargo build --locked --manifest-path "$CARGO_MANIFEST"
  BIN="$CARGO_TARGET_DIR/debug/task-registry-flow"
else
  check "cargo build (release)" cargo build --locked --release --manifest-path "$CARGO_MANIFEST"
  BIN="$CARGO_TARGET_DIR/release/task-registry-flow"
fi
[ -x "$BIN" ] || die "binary not found at $BIN"

echo ""
echo "--- Tests ---"
check "cargo test (95 tests)" cargo test --locked --manifest-path "$CARGO_MANIFEST"

echo ""
echo "--- CLI subcommands ---"
check_output "validate"              "task registry validate ok" "$BIN" validate
check_output "metrics"              "Task registry metrics"     "$BIN" metrics
check_output "source-limit check"   "source file limit ok"      "$BIN" source-limit check
check       "source-limit plan"                                 "$BIN" source-limit plan
check       "status-check"                                      "$BIN" status-check
check       "status-check --json"                               "$BIN" status-check --format json
check       "verify-mutation-hook"                              "$BIN" verify-mutation-hook
check       "verify-mutation-hook --codex"                      "$BIN" verify-mutation-hook --format codex
check       "verify-mutation-hook --antigravity"                "$BIN" verify-mutation-hook --format antigravity
check       "verify-mutation-hook --cursor"                     "$BIN" verify-mutation-hook --format cursor
check       "install plan"                                      "$BIN" install plan
check       "install plan --json"                               "$BIN" install plan --format json
check       "verify-chain"                                      "$BIN" verify-chain
check       "verify-chain --json"                               "$BIN" verify-chain --format json
check       "release-check all"                                 "$BIN" release-check all
check       "release-check all --json"                          "$BIN" release-check all --format json

echo ""
echo "--- Scripts ---"
check_any   "status.sh --strict"        "Summary:"              bash scripts/status.sh --strict
check       "test-install-modes.sh"                             bash scripts/test-install-modes.sh
check       "render-from-config merge"                          env MODE=merge DRY_RUN=1 bash scripts/render-from-config.sh

echo ""
echo "--- Wrapper ---"
if [ -x "$REPO_ROOT/.codex/scripts/task-registry" ]; then
  check_output ".codex/scripts/task-registry validate" "task registry validate ok" \
    "$REPO_ROOT/.codex/scripts/task-registry" validate
else
  echo -e "  ${YELLOW}SKIP${NC} .codex/scripts/task-registry not executable"
fi

echo ""
echo "---"
echo -e "Passed: ${GREEN}${pass}${NC}  Warnings: ${YELLOW}${warn}${NC}  Failed: ${RED}${fail}${NC}"
if [ "$fail" -gt 0 ]; then
  echo -e "${RED}SMOKE TEST FAILED${NC}"
  exit 1
else
  echo -e "${GREEN}SMOKE TEST PASSED${NC}"
  exit 0
fi
