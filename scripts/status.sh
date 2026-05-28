#!/usr/bin/env bash
# Report agent-governance install posture for the current repo.
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
TARGET_ROOT="$(cd "$TARGET_ROOT" && pwd)"
CONFIG="${PLUGIN_ROOT}/examples/spectrum-arcana.project.config.toml"
STRICT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=1; shift ;;
    -*) echo "unknown option: $1" >&2; exit 2 ;;
    *) CONFIG="$1"; shift ;;
  esac
done

pass=0
warn=0
fail=0
overlay_installed=0

ok()   { printf '  OK   %s\n' "$1"; pass=$((pass + 1)); }
note() { printf '  NOTE %s\n' "$1"; warn=$((warn + 1)); }
bad()  { printf '  FAIL %s\n' "$1"; fail=$((fail + 1)); }

check_markers() {
  local file="$1"
  if [[ ! -f "$file" ]]; then
    bad "$file missing"
    return
  fi
  local begin_count end_count
  begin_count="$(grep -c 'agent-governance:begin' "$file" || true)"
  end_count="$(grep -c 'agent-governance:end' "$file" || true)"
  if [[ "$begin_count" -eq 1 && "$end_count" -eq 1 ]]; then
    ok "$file overlay markers present (single pair)"
    overlay_installed=1
  elif [[ "$begin_count" -gt 0 || "$end_count" -gt 0 ]]; then
    bad "$file overlay markers malformed (begin=$begin_count end=$end_count)"
  else
    note "$file has no agent-governance overlay markers"
  fi
}

check_symlink() {
  local link="$1" expected_target="$2"
  if [[ ! -e "$link" ]]; then
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

check_file_contains() {
  local file="$1" needle="$2" label="$3"
  if [[ -f "$file" ]] && grep -q "$needle" "$file"; then
    ok "$label"
  else
    bad "$label"
  fi
}

skill_diff_hint() {
  local skill="$1"
  local live="${TARGET_ROOT}/.cursor/skills/${skill}/SKILL.md"
  local plugin="${PLUGIN_ROOT}/skills/${skill}/SKILL.md"
  if [[ ! -f "$live" ]]; then
    bad "skill missing: $live"
    return
  fi
  if [[ ! -f "$plugin" ]]; then
    note "plugin skill missing: $plugin"
    return
  fi
  if diff -q "$live" "$plugin" >/dev/null 2>&1; then
    ok "skill ${skill}/SKILL.md matches plugin portable base"
  else
    note "skill ${skill}/SKILL.md differs from plugin (expected when PROJECT.md extends)"
  fi
  if [[ -f "${live%/SKILL.md}/PROJECT.md" ]]; then
    ok "skill ${skill}/PROJECT.md present"
  elif [[ "$overlay_installed" -eq 1 || "$STRICT" -eq 1 ]]; then
    bad "skill ${skill}/PROJECT.md required when overlay is installed (--strict)"
  else
    note "skill ${skill}/PROJECT.md absent (optional without overlay)"
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

echo "Agent governance status"
echo "  repo:   ${TARGET_ROOT}"
echo "  plugin: ${PLUGIN_ROOT}"
echo "  config: ${CONFIG}"
echo "  requirements: ${PLUGIN_ROOT}/REQUIREMENTS.toml"
echo ""

echo "Overlay markers"
check_markers "${TARGET_ROOT}/AGENTS.md"
check_markers "${TARGET_ROOT}/GEMINI.md"
echo ""

echo "Plugin link"
check_symlink "${TARGET_ROOT}/.agents/plugins/agent-governance" "../../plugins/agent-governance"
echo ""

echo "Mutation gate"
check_hook_uses_env "${TARGET_ROOT}/tools/antigravity/pre-tool-use-gap-closure.sh"
check_file_contains "${TARGET_ROOT}/.codex/governance-cli.env" 'GOVERNANCE_VERIFY_HOOK_CMD' "governance-cli.env defines GOVERNANCE_VERIFY_HOOK_CMD"
if [[ -f "${TARGET_ROOT}/.agents/hooks.json" ]] && grep -q 'tools/antigravity/pre-tool-use-gap-closure.sh' "${TARGET_ROOT}/.agents/hooks.json"; then
  ok ".agents/hooks.json uses canonical hook script path"
else
  bad ".agents/hooks.json does not point at tools/antigravity/pre-tool-use-gap-closure.sh"
fi
if [[ -f "${PLUGIN_ROOT}/hooks.json" ]] && grep -q 'tools/antigravity/pre-tool-use-gap-closure.sh' "${PLUGIN_ROOT}/hooks.json"; then
  ok "plugin hooks.json uses canonical hook script path"
else
  bad "plugin hooks.json not aligned with canonical hook path"
fi
echo ""

echo "Skills"
check_symlink "${TARGET_ROOT}/.agents/skills/gap-closure-contract" "../../.cursor/skills/gap-closure-contract"
check_symlink "${TARGET_ROOT}/.agents/skills/task-registry-flow" "../../.cursor/skills/task-registry-flow"
skill_diff_hint "gap-closure-contract"
skill_diff_hint "task-registry-flow"
echo ""

if [[ "$STRICT" -eq 1 ]]; then
  echo "Tracked for CI (REQUIREMENTS.toml)"
  check_tracked_for_ci
  echo ""
fi

echo "Registry CLI"
if (cd "$TARGET_ROOT" && cargo run --quiet --bin task_registry -- validate >/dev/null 2>&1); then
  ok "cargo run --bin task_registry -- validate (includes agent governance posture)"
else
  bad "cargo run --bin task_registry -- validate failed"
fi

echo ""
printf 'Summary: %d ok, %d note, %d fail\n' "$pass" "$warn" "$fail"
if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
