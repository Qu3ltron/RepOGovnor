#!/usr/bin/env bash
# Report agent-governance install posture for the current repo.
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
TARGET_ROOT="$(cd "$TARGET_ROOT" && pwd)"
CONFIG=""
STRICT=0
ENV_FILTER="all"
RELEASE_SOURCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    --release-source) RELEASE_SOURCE=1; shift ;;
    --env) ENV_FILTER="$2"; shift 2 ;;
    -*) echo "unknown option: $1" >&2; exit 2 ;;
    *) CONFIG="$1"; shift ;;
  esac
done

if [[ -z "$CONFIG" ]]; then
  if [[ -f "${TARGET_ROOT}/project.config.toml" ]]; then
    CONFIG="${TARGET_ROOT}/project.config.toml"
  else
    CONFIG="${PLUGIN_ROOT}/examples/spectrum-arcana.project.config.toml"
  fi
fi

case "$ENV_FILTER" in
  all|codex|antigravity|cursor) ;;
  *) echo "unknown --env: $ENV_FILTER" >&2; exit 2 ;;
esac

pass=0
warn=0
fail=0
marker_installed=0
STATUS_CHECK_JSON=""
STATUS_CHECK_RAN=0

ok()   { printf '  OK   %s\n' "$1"; pass=$((pass + 1)); }
note() { printf '  NOTE %s\n' "$1"; warn=$((warn + 1)); }
bad()  { printf '  FAIL %s\n' "$1"; fail=$((fail + 1)); }

task_registry() {
  local wrapper="${TARGET_ROOT}/.codex/scripts/task-registry"
  if [[ -x "$wrapper" ]]; then
    (cd "$TARGET_ROOT" && "$wrapper" "$@")
    return
  fi

  local manifest="${PLUGIN_ROOT}/rust/task-registry-flow-cli/Cargo.toml"
  if [[ ! -f "$manifest" ]]; then
    manifest="${TARGET_ROOT}/rust/task-registry-flow-cli/Cargo.toml"
  fi
  (
    cd "$TARGET_ROOT"
    CARGO_TARGET_DIR="${AGENT_GOVERNANCE_CARGO_TARGET_DIR:-/tmp/agent-governance-cargo-target}" \
      cargo run --locked --quiet --manifest-path "$manifest" -- "$@"
  )
}

check_markers() {
  local file="$1"
  local rel result status actual
  rel="$(basename "$file")"
  run_status_check_json
  result="$(
    STATUS_CHECK_JSON="$STATUS_CHECK_JSON" STATUS_CHECK_PATH="$rel" python3 <<'PY'
import json
import os
payload = json.loads(os.environ["STATUS_CHECK_JSON"])
path = os.environ["STATUS_CHECK_PATH"]
for check in payload["checks"]:
    if check["check_id"] == "governance-marker" and check["path"] == path:
        print(f'{check["status"]}\t{check["actual"]}')
        break
else:
    print("fail\tmissing status diagnostic")
PY
  )"
  status="${result%%$'\t'*}"
  actual="${result#*$'\t'}"
  if [[ "$status" == "pass" ]]; then
    ok "$file governance markers present (single pair)"
    marker_installed=1
  elif [[ "$actual" == "missing marker block" ]]; then
    bad "$file missing governance marker block"
  else
    bad "$file governance markers malformed (${actual})"
  fi
}

run_status_check_json() {
  if [[ "$STATUS_CHECK_RAN" -eq 1 ]]; then
    return
  fi
  STATUS_CHECK_RAN=1
  local output
  if output="$(task_registry status-check --format json 2>/tmp/agent-governance-status-check.err)"; then
    STATUS_CHECK_JSON="$output"
  else
    STATUS_CHECK_JSON="$output"
    if [[ -z "$STATUS_CHECK_JSON" ]]; then
      STATUS_CHECK_JSON='{"checks":[]}'
    fi
  fi
}

check_symlink() {
  local link="$1" expected_target="$2"
  if [[ ! -e "$link" && ! -L "$link" ]]; then
    bad "missing symlink: $link"
    return
  fi
  if [[ -L "$link" ]]; then
    local actual resolved
    actual="$(readlink "$link")"
    if [[ "$actual" == "$expected_target" ]]; then
      resolved="$(cd "$(dirname "$link")" && cd "$actual" && pwd)"
      if [[ -d "$resolved" ]]; then
        ok "symlink $link -> $expected_target (resolves)"
      else
        bad "symlink $link -> $expected_target does not resolve to a directory"
      fi
    else
      bad "symlink $link -> $actual (expected $expected_target)"
    fi
  else
    bad "$link exists but is not a symlink"
  fi
}

check_absent() {
  local path="$1" label="$2"
  if [[ -e "$path" || -L "$path" ]]; then
    bad "$label"
  else
    ok "$label"
  fi
}

check_hook_uses_env() {
  local script="$1"
  if [[ ! -f "$script" ]]; then
    bad "hook script missing: $script"
    return
  fi
  if grep -q 'governance-cli.env' "$script" && grep -q 'GOVERNANCE_VERIFY_HOOK_CMD' "$script"; then
    ok "hook script reads .codex/governance-cli.env"
  else
    bad "hook script does not source governance-cli.env"
  fi
}

check_plugin_checkout() {
  local path="${TARGET_ROOT}/plugins/agent-governance"
  if [[ ! -e "$path" ]]; then
    bad "plugins/agent-governance missing; install the plugin as a repo-local submodule or vendored checkout"
    return
  fi
  if [[ -L "$path" ]]; then
    local target resolved
    target="$(readlink "$path")"
    if [[ "$target" == /* ]]; then
      bad "plugins/agent-governance is an absolute symlink; CI requires a repo-local submodule or vendored checkout"
      return
    fi
    resolved="$(cd "$(dirname "$path")" && cd "$target" && pwd)"
    case "${resolved}/" in
      "${TARGET_ROOT}/"*) ;;
      *)
        bad "plugins/agent-governance symlink resolves outside the repo; CI requires a repo-local submodule or vendored checkout"
        return
        ;;
    esac
  fi
  if [[ -f "${path}/.codex-plugin/plugin.json" ]] && [[ -x "${path}/scripts/status.sh" ]]; then
    ok "plugins/agent-governance repo-local checkout present"
  else
    bad "plugins/agent-governance is missing plugin metadata or executable status.sh"
  fi
}

check_file_contains() {
  local file="$1" needle="$2" label="$3"
  if [[ -f "$file" ]] && grep -q "$needle" "$file"; then
    ok "$label"
  else
    bad "$label"
  fi
}

check_executable() {
  local path="$1" label="$2"
  if [[ -x "$path" ]]; then
    ok "$label"
  else
    bad "$label"
  fi
}

version_at_least() {
  local actual="$1" minimum="$2"
  [[ "$(printf '%s\n%s\n' "$minimum" "$actual" | sort -V | head -n1)" == "$minimum" ]]
}

skill_diff_hint() {
  local skill="$1" base="$2"
  local skill_dir="${TARGET_ROOT}/${base}/${skill}"
  local live="${skill_dir}/SKILL.md"
  local plugin="${PLUGIN_ROOT}/skills/${skill}/SKILL.md"
  if [[ -L "$skill_dir" ]]; then
    bad "${base}/${skill} must be a native directory, not a symlink"
    return
  fi
  if [[ ! -f "$live" ]]; then
    bad "skill missing: $live"
    return
  fi
  if [[ ! -f "$plugin" ]]; then
    note "plugin skill missing: $plugin"
    return
  fi
  if diff -q "$live" "$plugin" >/dev/null 2>&1; then
    ok "${base}/${skill}/SKILL.md matches plugin portable base"
  else
    note "${base}/${skill}/SKILL.md differs from plugin (expected when PROJECT.md extends)"
  fi
  if [[ -f "${live%/SKILL.md}/PROJECT.md" ]]; then
    ok "skill ${skill}/PROJECT.md present"
  elif [[ "$marker_installed" -eq 1 || "$STRICT" -eq 1 ]]; then
    bad "skill ${skill}/PROJECT.md required when governance markers are installed (--strict)"
  else
    note "skill ${skill}/PROJECT.md absent (optional without governance markers)"
  fi
}

check_tracked_for_ci() {
  if [[ ! -d "${TARGET_ROOT}/.git" ]]; then
    note "not a git repo; skipping tracked-for-CI checks"
    return
  fi
  local path
  while IFS= read -r path; do
    [[ -z "$path" ]] && continue
    if [[ ! -e "${TARGET_ROOT}/${path}" ]]; then
      bad "missing required CI artifact: ${path}"
      continue
    fi
    if [[ "$path" == "plugins/agent-governance" ]]; then
      local tracked stage
      tracked="$(cd "$TARGET_ROOT" && git ls-files "$path")"
      if [[ -n "$tracked" ]]; then
        stage="$(cd "$TARGET_ROOT" && git ls-files --stage -- "$path")"
        if [[ "$stage" == 160000* ]]; then
          if [[ -f "${TARGET_ROOT}/.gitmodules" ]] \
            && grep -q 'path = plugins/agent-governance' "${TARGET_ROOT}/.gitmodules"; then
            ok "git tracks ${path} as a configured submodule"
          else
            bad "${path} is an embedded gitlink without .gitmodules; use git submodule add or vendor files without .git"
          fi
        else
          ok "git tracks ${path}"
        fi
      else
        bad "not tracked in git: ${path} (required for CI — use submodule or vendored checkout)"
      fi
      continue
    fi
    if (cd "$TARGET_ROOT" && git ls-files --error-unmatch "$path" >/dev/null 2>&1); then
      ok "git tracks ${path}"
    else
      bad "not tracked in git: ${path} (required for CI — see ${PLUGIN_ROOT}/REQUIREMENTS.toml)"
    fi
  done < <(
    PLUGIN_ROOT="${PLUGIN_ROOT}" python3 <<'PY'
import os
import tomllib
from pathlib import Path
plugin_root = Path(os.environ["PLUGIN_ROOT"])
req = tomllib.loads((plugin_root / "REQUIREMENTS.toml").read_text(encoding="utf-8"))
for path in req["tracked_for_ci"]["required"]:
    print(path)
PY
  )
}

check_release_file() {
  local path="$1"
  if [[ -f "${TARGET_ROOT}/${path}" ]]; then
    ok "release file present: ${path}"
  else
    bad "release file missing: ${path}"
  fi
}

check_release_absent() {
  local path="$1"
  if [[ -e "${TARGET_ROOT}/${path}" || -L "${TARGET_ROOT}/${path}" ]]; then
    bad "stale release-incompatible path present: ${path}"
  else
    ok "stale release-incompatible path absent: ${path}"
  fi
}

check_release_clean_git() {
  if [[ "${AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK:-0}" == "1" ]]; then
    require_local_waiver "dirty release-source check" "${AGENT_GOVERNANCE_DIRTY_RELEASE_CHECK_REASON:-}"
    return
  fi
  local status
  status="$(cd "$TARGET_ROOT" && git status --short)"
  if [[ -z "$status" ]]; then
    ok "git worktree clean for release"
  else
    bad "git worktree dirty; commit or discard changes before release"
    printf '%s\n' "$status" | sed 's/^/       /'
  fi
}

check_release_tracked() {
  if [[ "${AGENT_GOVERNANCE_ALLOW_DIRTY_RELEASE_CHECK:-0}" == "1" ]]; then
    require_local_waiver "tracked release-source check" "${AGENT_GOVERNANCE_DIRTY_RELEASE_CHECK_REASON:-}"
    return
  fi
  local path
  for path in "$@"; do
    if (cd "$TARGET_ROOT" && git ls-files --error-unmatch "$path" >/dev/null 2>&1); then
      ok "git tracks release file: ${path}"
    else
      bad "release file not tracked in git: ${path}"
    fi
  done
}

require_local_waiver() {
  local label="$1" reason="$2"
  if [[ "${AGENT_GOVERNANCE_FINAL_RELEASE:-0}" == "1" ]]; then
    bad "${label} waiver forbidden in final release mode"
  elif [[ -z "$reason" ]]; then
    bad "${label} waiver requires a non-empty reason"
  else
    note "${label} waiver active: ${reason}"
  fi
}

run_release_source_status() {
  local release_files=()
  local stale_files=()
  mapfile -t release_files < <(
    PLUGIN_ROOT="${PLUGIN_ROOT}" python3 <<'PY'
import os
import tomllib
from pathlib import Path
req = tomllib.loads((Path(os.environ["PLUGIN_ROOT"]) / "REQUIREMENTS.toml").read_text(encoding="utf-8"))
for path in req["release_source"]["required"]:
    print(path)
PY
  )
  mapfile -t stale_files < <(
    PLUGIN_ROOT="${PLUGIN_ROOT}" python3 <<'PY'
import os
import tomllib
from pathlib import Path
req = tomllib.loads((Path(os.environ["PLUGIN_ROOT"]) / "REQUIREMENTS.toml").read_text(encoding="utf-8"))
for path in req["release_source"].get("stale_absent", []):
    print(path)
PY
  )

  echo "Agent governance release-source status"
  echo "  repo:   ${TARGET_ROOT}"
  echo "  plugin: ${PLUGIN_ROOT}"
  echo ""

  echo "Release artifacts"
  local path
  for path in "${release_files[@]}"; do
    check_release_file "$path"
  done
  for path in "${stale_files[@]}"; do
    check_release_absent "$path"
  done
  echo ""

  echo "Release version"
  if (cd "$TARGET_ROOT" && scripts/release-version-check.sh >/dev/null); then
    ok "release versions are consistent"
  else
    bad "release versions are inconsistent"
  fi
  check_file_contains "${TARGET_ROOT}/CHANGELOG.md" '## 2.0.0' "CHANGELOG.md includes 2.0.0 section"
  check_file_contains "${TARGET_ROOT}/LICENSE" 'MIT License' "LICENSE is MIT"
  check_file_contains "${TARGET_ROOT}/docs/releases/v2.md" 'Audit Policy' "v2 release docs include audit policy"
  check_file_contains "${TARGET_ROOT}/docs/releases/v2.md" 'License: MIT' "v2 release docs state MIT license"
  check_file_contains "${TARGET_ROOT}/README.md" 'CHANGELOG.md' "README links changelog"
  check_file_contains "${TARGET_ROOT}/README.md" 'docs/releases/v2.md' "README links v2 release checklist"
  check_file_contains "${TARGET_ROOT}/README.md" 'VISION.md' "README links vision"
  check_file_contains "${TARGET_ROOT}/README.md" 'ROADMAP.md' "README links roadmap"
  check_file_contains "${TARGET_ROOT}/README.md" 'License: MIT' "README states MIT license"
  echo ""

  echo "Package validation"
  if task_registry source-limit check >/dev/null; then
    ok "source limit check"
  else
    bad "source limit check failed"
  fi
  if task_registry validate >/dev/null; then
    ok "task registry validate"
  else
    bad "task registry validate failed"
  fi
  if task_registry release-check all --format json >/dev/null; then
    ok "schema-backed release check"
  else
    bad "schema-backed release check failed"
  fi
  if [[ "${AGENT_GOVERNANCE_ALLOW_ACTIVE_RELEASE_TASKS:-0}" == "1" ]]; then
    require_local_waiver "active release task" "${AGENT_GOVERNANCE_ACTIVE_RELEASE_TASKS_REASON:-}"
  elif task_registry metrics | grep -q 'active=0'; then
    ok "task registry has no active tasks"
  else
    bad "task registry still has active tasks"
  fi
  if task_registry metrics | grep -q 'deferred=0, blocked=0'; then
    ok "task registry has no deferred or blocked tasks"
  else
    bad "task registry has deferred or blocked tasks"
  fi
  if ! command -v agy >/dev/null 2>&1; then
    note "agy CLI not found on PATH; skip runtime plugin validation"
  elif (cd "$TARGET_ROOT" && agy plugin validate . >/dev/null 2>&1); then
    ok "agy plugin validate"
  else
    bad "agy plugin validate failed"
  fi
  echo ""

  echo "Git release hygiene"
  check_release_clean_git
  check_release_tracked "${release_files[@]}"
  echo ""

  printf 'Summary: %d ok, %d note, %d fail\n' "$pass" "$warn" "$fail"
  if [[ "$fail" -gt 0 ]]; then
    exit 1
  fi
}

if [[ "$RELEASE_SOURCE" -eq 1 ]]; then
  run_release_source_status
  exit 0
fi

echo "Agent governance status"
echo "  repo:   ${TARGET_ROOT}"
echo "  plugin: ${PLUGIN_ROOT}"
echo "  config: ${CONFIG}"
echo "  requirements: ${PLUGIN_ROOT}/REQUIREMENTS.toml"
echo ""

echo "Governance markers"
check_markers "${TARGET_ROOT}/AGENTS.md"
check_markers "${TARGET_ROOT}/GEMINI.md"
echo ""

echo "Plugin link"
check_plugin_checkout
check_symlink "${TARGET_ROOT}/.agents/plugins/agent-governance" "../../plugins/agent-governance"
if [[ -f "${PLUGIN_ROOT}/.codex-plugin/plugin.json" ]]; then
  ok "plugin .codex-plugin/plugin.json present"
else
  bad "plugin .codex-plugin/plugin.json missing"
fi
echo ""

echo "Mutation gate"
check_hook_uses_env "${TARGET_ROOT}/tools/agent-governance/pre-tool-use-gap-closure.sh"
check_file_contains "${TARGET_ROOT}/.codex/governance-cli.env" 'GOVERNANCE_VERIFY_HOOK_CMD' "governance-cli.env defines GOVERNANCE_VERIFY_HOOK_CMD"
check_file_contains "${TARGET_ROOT}/.codex/governance-cli.env" '.codex/scripts/task-registry verify-mutation-hook' "governance-cli.env uses plugin-owned mutation hook"
if [[ -f "${TARGET_ROOT}/.agents/hooks.json" ]] && grep -q 'tools/agent-governance/pre-tool-use-gap-closure.sh' "${TARGET_ROOT}/.agents/hooks.json"; then
  ok ".agents/hooks.json uses canonical hook script path"
else
  bad ".agents/hooks.json does not point at tools/agent-governance/pre-tool-use-gap-closure.sh"
fi
if [[ -f "${TARGET_ROOT}/.cursor/hooks.json" ]] && grep -q 'beforeShellExecution' "${TARGET_ROOT}/.cursor/hooks.json"; then
  ok ".cursor/hooks.json defines Cursor shell/file guardrails"
else
  bad ".cursor/hooks.json missing Cursor guardrail hooks"
fi
check_hook_uses_env "${TARGET_ROOT}/.cursor/hooks/gap-closure-gate.sh"
if [[ -f "${TARGET_ROOT}/.cursor/hooks/gap-closure-gate.sh" ]] \
  && grep -q 'GOVERNANCE_HOOK_FORMAT=cursor' "${TARGET_ROOT}/.cursor/hooks/gap-closure-gate.sh" \
  && grep -q 'tools/agent-governance/pre-tool-use-gap-closure.sh' "${TARGET_ROOT}/.cursor/hooks/gap-closure-gate.sh"; then
  ok ".cursor/hooks/gap-closure-gate.sh delegates to canonical mutation gate"
else
  bad ".cursor/hooks/gap-closure-gate.sh must set GOVERNANCE_HOOK_FORMAT=cursor and delegate to canonical script"
fi
if [[ -f "${PLUGIN_ROOT}/hooks/hooks.json" ]] && grep -q 'pre-tool-use-gap-closure.sh' "${PLUGIN_ROOT}/hooks/hooks.json"; then
  ok "Antigravity plugin hooks/hooks.json present"
else
  bad "Antigravity plugin hooks/hooks.json missing"
fi
if [[ -f "${PLUGIN_ROOT}/hooks/codex-hooks.json" ]] && grep -q 'GOVERNANCE_HOOK_FORMAT=codex' "${PLUGIN_ROOT}/hooks/codex-hooks.json"; then
  ok "Codex plugin hooks/codex-hooks.json present"
else
  bad "Codex plugin hooks/codex-hooks.json missing"
fi
echo ""

echo "Skills"
skill_diff_hint "gap-closure-contract" ".agents/skills"
skill_diff_hint "task-registry-flow" ".agents/skills"
skill_diff_hint "gap-closure-contract" ".cursor/skills"
skill_diff_hint "task-registry-flow" ".cursor/skills"
if [[ -f "${TARGET_ROOT}/.agents/skills/gap-closure-contract.md" ]] && [[ -f "${TARGET_ROOT}/.agents/skills/task-registry-flow.md" ]]; then
  ok "Antigravity markdown skills present"
else
  bad "Antigravity markdown skills missing"
fi
echo ""

echo "Task registry artifacts"
check_executable "${TARGET_ROOT}/.codex/scripts/task-registry" ".codex/scripts/task-registry is executable"
check_file_contains "${TARGET_ROOT}/.codex/scripts/task-registry" 'task-registry-flow-cli/Cargo.toml' "task-registry wrapper points at plugin Rust CLI"
check_file_contains "${TARGET_ROOT}/.codex/config.toml" 'hooks = true' ".codex/config.toml enables Codex hooks"
check_file_contains "${TARGET_ROOT}/.codex/hooks.json" 'PreToolUse' ".codex/hooks.json defines Codex PreToolUse hook"
check_file_contains "${TARGET_ROOT}/.codex/agent-governance.toml" 'cli_command = ".codex/scripts/task-registry"' ".codex/agent-governance.toml uses plugin-owned registry CLI"
check_absent "${TARGET_ROOT}/.codex/settings.toml" "stale .codex/settings.toml absent"
check_absent "${TARGET_ROOT}/.codex/hooks/user-plan-approval.toml" "stale Codex hook TOML absent"
check_absent "${TARGET_ROOT}/.gemini/settings.json" "stale workspace .gemini/settings.json absent"
check_absent "${TARGET_ROOT}/hooks.json" "stale root hooks.json absent"
check_absent "${TARGET_ROOT}/tools/antigravity/pre-tool-use-gap-closure.sh" "stale Antigravity hook path absent"
if [[ -f "${TARGET_ROOT}/docs/task-registry.toml" ]]; then
  ok "docs/task-registry.toml present"
else
  bad "docs/task-registry.toml missing"
fi
if [[ -f "${TARGET_ROOT}/docs/task-registry/events.jsonl" ]]; then
  ok "docs/task-registry/events.jsonl present"
else
  bad "docs/task-registry/events.jsonl missing"
fi
if [[ -f "${TARGET_ROOT}/.codex/templates/task-registry-plan-template.md" ]]; then
  ok ".codex/templates/task-registry-plan-template.md present"
else
  bad ".codex/templates/task-registry-plan-template.md missing"
fi
if [[ -f "${TARGET_ROOT}/.github/workflows/agent-governance.yml" ]]; then
  ok ".github/workflows/agent-governance.yml present"
else
  bad ".github/workflows/agent-governance.yml missing"
fi
echo ""

if [[ "$ENV_FILTER" == "all" || "$ENV_FILTER" == "codex" ]]; then
  echo "Codex environment"
  if command -v codex >/dev/null 2>&1; then
    ok "codex CLI present: $(codex --version 2>/dev/null | head -1)"
  else
    note "codex CLI not found on PATH"
  fi
  check_file_contains "${TARGET_ROOT}/.codex/hooks.json" 'GOVERNANCE_HOOK_FORMAT=codex' "Codex hook uses codex output format"
  check_file_contains "${TARGET_ROOT}/.codex/config.toml" 'hooks = true' "Codex project config enables hooks"
  echo ""
fi

if [[ "$ENV_FILTER" == "all" || "$ENV_FILTER" == "antigravity" ]]; then
  echo "Antigravity environment"
  if command -v agy >/dev/null 2>&1; then
    agy_version="$(agy --version 2>/dev/null | head -1)"
    ok "agy CLI present: ${agy_version}"
    if version_at_least "$agy_version" "1.0.3"; then
      ok "agy CLI meets minimum 1.0.3"
    else
      bad "agy CLI ${agy_version} is older than required 1.0.3"
    fi
    if agy plugin validate "$PLUGIN_ROOT" 2>/tmp/agent-governance-agy-validate.log | grep -q 'hooks.*processed'; then
      ok "agy plugin validate processes hooks"
    else
      bad "agy plugin validate did not process hooks"
    fi
  else
    note "agy CLI not found on PATH; skip runtime plugin validation"
  fi
  check_file_contains "${TARGET_ROOT}/.agents/hooks.json" 'edit_file' "Antigravity hook matcher includes edit_file"
  echo ""
fi

if [[ "$ENV_FILTER" == "all" || "$ENV_FILTER" == "cursor" ]]; then
  echo "Cursor environment"
  if command -v cursor-agent >/dev/null 2>&1; then
    ok "cursor-agent present: $(cursor-agent --version 2>/dev/null | head -1)"
  else
    note "cursor-agent not found on PATH"
  fi
  check_file_contains "${TARGET_ROOT}/.cursor/hooks.json" 'beforeShellExecution' "Cursor hooks include beforeShellExecution"
  check_file_contains "${TARGET_ROOT}/.cursor/rules/agent-governance.mdc" '1600 lines' "Cursor rule embeds source limit"
  echo ""
fi

if [[ "$STRICT" -eq 1 ]]; then
  echo "Tracked for CI (REQUIREMENTS.toml)"
  check_tracked_for_ci
  echo ""
fi

echo "Registry CLI"
if (cd "$TARGET_ROOT" && .codex/scripts/task-registry validate >/dev/null 2>&1); then
  ok ".codex/scripts/task-registry validate"
else
  bad ".codex/scripts/task-registry validate failed"
fi
if (cd "$TARGET_ROOT" && .codex/scripts/task-registry source-limit check >/dev/null 2>&1); then
  ok ".codex/scripts/task-registry source-limit check"
else
  bad ".codex/scripts/task-registry source-limit check failed"
fi

echo ""
printf 'Summary: %d ok, %d note, %d fail\n' "$pass" "$warn" "$fail"
if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
