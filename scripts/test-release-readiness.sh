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
  cp "$ROOT/REQUIREMENTS.toml" "$tmp/REQUIREMENTS.toml"
  cp "$ROOT/plugin.json" "$tmp/plugin.json"
  cp "$ROOT/.codex-plugin/plugin.json" "$tmp/.codex-plugin/plugin.json"
  cp "$ROOT/MANIFEST.toml" "$tmp/MANIFEST.toml"
  cp "$ROOT/rust/task-registry-flow-cli/Cargo.lock" "$tmp/rust/task-registry-flow-cli/Cargo.lock"
  cp "$ROOT/rust/task-registry-flow-cli/Cargo.toml" "$tmp/rust/task-registry-flow-cli/Cargo.toml"
  cp -R "$ROOT/rust/task-registry-flow-cli/src" "$tmp/rust/task-registry-flow-cli/src"
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
  grep -q 'release-version-consistent' /tmp/release-version-drift.out

  local missing
  missing="$(tmp_copy)"
  rm "$missing/VERSION"
  if "$ROOT/scripts/release-version-check.sh" "$missing" >/tmp/release-version-missing.out 2>&1; then
    echo "missing VERSION unexpectedly passed" >&2
    exit 1
  fi
  grep -q 'VERSION' /tmp/release-version-missing.out

  local bad_schema
  bad_schema="$(tmp_copy)"
  perl -0pi -e 's/format = "plain"/format = "bogus"/' "$bad_schema/REQUIREMENTS.toml"
  if "$ROOT/scripts/release-version-check.sh" "$bad_schema" >/tmp/release-version-schema.out 2>&1; then
    echo "unknown version file schema unexpectedly passed" >&2
    exit 1
  fi
  grep -q 'unknown variant' /tmp/release-version-schema.out
}

check_artifacts() {
  test -f "$ROOT/CHANGELOG.md"
  test -f "$ROOT/LICENSE"
  test -f "$ROOT/docs/releases/v2.md"
  test -f "$ROOT/rust/task-registry-flow-cli/deny.toml"
  grep -q 'MIT License' "$ROOT/LICENSE"
  grep -q '2.0.0' "$ROOT/CHANGELOG.md"
  grep -q 'Breaking changes' "$ROOT/CHANGELOG.md"
  grep -q 'License: MIT' "$ROOT/README.md"
  grep -q 'Audit Policy' "$ROOT/docs/releases/v2.md"
  grep -q 'License: MIT' "$ROOT/docs/releases/v2.md"
  grep -q 'Unicode-3.0' "$ROOT/rust/task-registry-flow-cli/deny.toml"
  grep -q 'ignore = true' "$ROOT/rust/task-registry-flow-cli/deny.toml"
  grep -q 'CHANGELOG.md' "$ROOT/README.md"
  grep -q 'docs/releases/v2.md' "$ROOT/README.md"
  grep -q 'VISION.md' "$ROOT/README.md"
  grep -q 'ROADMAP.md' "$ROOT/README.md"
  grep -q 'Primary users' "$ROOT/VISION.md"
  grep -q 'Known gaps' "$ROOT/ROADMAP.md"
  grep -q 'Runtime Schemas' "$ROOT/docs/runtime-schemas.md"
  grep -q 'agent-governance:begin' "$ROOT/templates/AGENTS.md.template"
  grep -q 'agent-governance:end' "$ROOT/templates/AGENTS.md.template"
  grep -q 'agent-governance:begin' "$ROOT/templates/GEMINI.md.template"
  grep -q 'agent-governance:end' "$ROOT/templates/GEMINI.md.template"
  grep -q 'agent-governance:begin' "$ROOT/AGENTS.md"
  grep -q 'agent-governance:begin' "$ROOT/GEMINI.md"
}

check_executable() {
  test -x "$ROOT/scripts/install-to-workspace.sh"
  test -x "$ROOT/scripts/render-from-config.sh"
  test -x "$ROOT/scripts/status.sh"
  test -x "$ROOT/scripts/test-install-modes.sh"
  test -x "$ROOT/scripts/release-version-check.sh"
  test -x "$ROOT/scripts/release-audit.sh"
  test -x "$ROOT/scripts/test-release-readiness.sh"

  "$ROOT/.codex/scripts/task-registry" release-check all --format json >/tmp/release-executable-positive.out
  python3 - <<'PY'
import json
from pathlib import Path
payload = json.loads(Path("/tmp/release-executable-positive.out").read_text(encoding="utf-8"))
assert payload["surface"] == "release-source"
assert payload["summary"]["fail"] == 0
assert any(
    check["check_id"] == "release-file-executable"
    and check["path"] == "scripts/test-install-modes.sh"
    and check["status"] == "pass"
    for check in payload["checks"]
)
PY

  local executable_copy
  executable_copy="$(mktemp -d)"
  CLEANUP_PATHS+=("$executable_copy")
  tar -C "$ROOT" \
    --exclude='.git' \
    --exclude='target' \
    --exclude='rust/target' \
    --exclude='.release-readiness-nested' \
    -cf - . | tar -C "$executable_copy" -xf -
  chmod 0644 "$executable_copy/scripts/test-install-modes.sh"
  if (
    cd "$executable_copy"
    cargo run --locked --quiet --manifest-path "$ROOT/rust/task-registry-flow-cli/Cargo.toml" -- \
      release-check all --format json > /tmp/release-executable-negative.out 2> /tmp/release-executable-negative.err
  ); then
    echo "non-executable release script unexpectedly passed" >&2
    exit 1
  fi
  python3 - <<'PY'
import json
from pathlib import Path
payload = json.loads(Path("/tmp/release-executable-negative.out").read_text(encoding="utf-8"))
assert payload["surface"] == "release-source"
assert payload["summary"]["fail"] >= 1
assert any(
    check["check_id"] == "release-file-executable"
    and check["path"] == "scripts/test-install-modes.sh"
    and check["status"] == "fail"
    and check["actual"] == "not executable"
    for check in payload["checks"]
)
PY
}

check_installer_schema() {
  local target config
  target="$(mktemp -d)"
  config="$(mktemp)"
  CLEANUP_PATHS+=("$target" "$config")
  git init -q "$target"
  cp "$ROOT/project.config.example.toml" "$config"

  MODE=force DRY_RUN=1 DRY_RUN_FORMAT=json \
    "$ROOT/scripts/render-from-config.sh" "$config" "$target" >/tmp/installer-dry-run-json.out
  python3 - <<'PY'
import json
from pathlib import Path
payload = json.loads(Path("/tmp/installer-dry-run-json.out").read_text(encoding="utf-8"))
assert payload["schema_version"] == 1
assert payload["surface"] == "installer-dry-run"
assert any(item["action"].startswith("would-") for item in payload["actions"])
PY

  printf '\n[unknown_runtime]\nloose = true\n' >> "$config"
  if MODE=force DRY_RUN=1 "$ROOT/scripts/render-from-config.sh" "$config" "$target" >/tmp/installer-unknown-config.out 2>&1; then
    echo "unknown installer config unexpectedly passed" >&2
    exit 1
  fi
  grep -q 'unknown project.config.toml section' /tmp/installer-unknown-config.out

  cp "$ROOT/project.config.example.toml" "$config"
  perl -0pi -e 's#cli_command = ".codex/scripts/task-registry"#cli_command = "scripts/task-registry"#' "$config"
  if MODE=force DRY_RUN=1 "$ROOT/scripts/render-from-config.sh" "$config" "$target" >/tmp/installer-noncanonical-config.out 2>&1; then
    echo "noncanonical installer config unexpectedly passed" >&2
    exit 1
  fi
  grep -q 'noncanonical project.config.toml' /tmp/installer-noncanonical-config.out
}

check_json_failure_reports() {
  local source_root
  source_root="$(mktemp -d)"
  CLEANUP_PATHS+=("$source_root")
  mkdir -p "$source_root/src"
  python3 - "$source_root/src/too-large.rs" <<'PY'
from pathlib import Path
import sys
Path(sys.argv[1]).write_text("fn item() {}\n" * 1601, encoding="utf-8")
PY
  if (
    cd "$source_root"
    cargo run --locked --quiet --manifest-path "$ROOT/rust/task-registry-flow-cli/Cargo.toml" -- \
      source-limit check --format json > /tmp/source-limit-json-fail.out 2> /tmp/source-limit-json-fail.err
  ); then
    echo "source-limit JSON failure unexpectedly exited zero" >&2
    exit 1
  fi
  python3 - <<'PY'
import json
from pathlib import Path
body = Path("/tmp/source-limit-json-fail.out").read_text(encoding="utf-8")
payload = json.loads(body)
assert payload["surface"] == "source-limit"
assert payload["summary"]["fail"] >= 1
assert any(check["path"] == "src/too-large.rs" and check["status"] == "fail" for check in payload["checks"])
PY

  local release_root
  release_root="$(tmp_copy)"
  CLEANUP_PATHS+=("$release_root")
  rm "$release_root/VERSION"
  if (
    cd "$release_root"
    cargo run --locked --quiet --manifest-path "$ROOT/rust/task-registry-flow-cli/Cargo.toml" -- \
      release-check all --format json > /tmp/release-json-fail.out 2> /tmp/release-json-fail.err
  ); then
    echo "release-check JSON failure unexpectedly exited zero" >&2
    exit 1
  fi
  python3 - <<'PY'
import json
from pathlib import Path
body = Path("/tmp/release-json-fail.out").read_text(encoding="utf-8")
assert not body.startswith("task-registry-flow error:")
payload = json.loads(body)
assert payload["surface"] == "release-source"
assert payload["summary"]["fail"] >= 1
assert any(check["check_id"] == "release-file-present" and check["path"] == "VERSION" and check["status"] == "fail" for check in payload["checks"])
PY
}

check_status() {
  AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK=1 \
    AGENT_GOVERNANCE_ALLOW_ACTIVE_RELEASE_TASKS=1 \
    "$ROOT/scripts/status.sh" --release-source
  if "$ROOT/scripts/status.sh" --strict >/tmp/release-consumer-strict.out 2>&1; then
    grep -q '0 fail' /tmp/release-consumer-strict.out
  else
    grep -q 'missing required CI artifact' /tmp/release-consumer-strict.out
  fi
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

  if "$ROOT/scripts/release-audit.sh" --root "$ROOT" >/tmp/release-audit-unknown-arg.out 2>&1; then
    echo "release audit unexpectedly accepted an ignored argument" >&2
    exit 1
  fi
  grep -q 'usage: release-audit.sh' /tmp/release-audit-unknown-arg.out
}

case "$MODE" in
  all)
    check_version
    check_artifacts
    check_executable
    check_installer_schema
    check_json_failure_reports
    check_status
    check_audit
    ;;
  version) check_version ;;
  artifacts) check_artifacts ;;
  executable) check_executable ;;
  installer) check_installer_schema ;;
  json-failures) check_json_failure_reports ;;
  status) check_status ;;
  audit) check_audit ;;
  *) echo "usage: test-release-readiness.sh [all|version|artifacts|executable|installer|json-failures|status|audit]" >&2; exit 2 ;;
esac

echo "release readiness tests ok: $MODE"
