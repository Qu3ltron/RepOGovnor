#!/usr/bin/env bash
# Canonical mutation gate: sources .codex/governance-cli.env (GOVERNANCE_VERIFY_HOOK_CMD).
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

format="${GOVERNANCE_HOOK_FORMAT:-antigravity}"
if [[ "${1:-}" == "--format" ]]; then
  format="${2:-antigravity}"
fi
case "$format" in
  antigravity|codex|cursor|claude) ;;
  *) format="antigravity" ;;
esac

emit_json() {
  local mode="$1"
  local reason="${2:-}"
  local escaped_reason
  escaped_reason="$(python3 -c 'import json, sys; print(json.dumps(sys.argv[1]))' "$reason" 2>/dev/null)" || {
    escaped_reason="$(printf '%s' "$reason" | tr '\r\n\t' '   ' | sed 's/\\/\\\\/g; s/"/\\"/g')"
    escaped_reason="\"${escaped_reason}\""
  }
  case "$format:$mode" in
    codex:deny|claude:deny)
      printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$escaped_reason"
      ;;
    codex:allow|claude:allow)
      ;;
    cursor:deny)
      printf '{"permission":"deny","user_message":%s,"agent_message":%s}\n' "$escaped_reason" "$escaped_reason"
      ;;
    cursor:allow)
      printf '{"permission":"allow"}\n'
      ;;
    *:deny)
      printf '{"decision":"deny","reason":%s}\n' "$escaped_reason"
      ;;
    *)
      printf '{"decision":"allow"}\n'
      ;;
  esac
}

emit_deny() {
  local reason="$1"
  emit_json deny "$reason"
}

canonical_verify_cmd=".codex/scripts/task-registry verify-mutation-hook"
base_verify_cmd="${GOVERNANCE_VERIFY_HOOK_CMD:-$canonical_verify_cmd}"
placeholder="{{"
placeholder="${placeholder}VERIFY_HOOK_COMMAND}}"

load_governance_env() {
  local env_path=".codex/governance-cli.env"
  local line value
  [[ -f "$env_path" ]] || return 0
  while IFS= read -r line || [[ -n "$line" ]]; do
    case "$line" in
      ""|"#"*) continue ;;
      GOVERNANCE_VERIFY_HOOK_CMD=*)
        value="${line#GOVERNANCE_VERIFY_HOOK_CMD=}"
        value="${value%\"}"
        value="${value#\"}"
        value="${value%\'}"
        value="${value#\'}"
        base_verify_cmd="$value"
        ;;
      *)
        emit_deny "invalid governance-cli.env content; run plugins/agent-governance/scripts/install-to-workspace.sh --merge"
        exit 0
        ;;
    esac
  done < "$env_path"
}

load_governance_env

if [[ -z "$base_verify_cmd" || "$base_verify_cmd" == *"$placeholder"* ]]; then
  emit_deny "GOVERNANCE_VERIFY_HOOK_CMD not configured; run plugins/agent-governance/scripts/install-to-workspace.sh --merge or set .codex/governance-cli.env"
  exit 0
fi

if [[ "$base_verify_cmd" != "$canonical_verify_cmd" ]]; then
  emit_deny "noncanonical GOVERNANCE_VERIFY_HOOK_CMD; run plugins/agent-governance/scripts/install-to-workspace.sh --merge"
  exit 0
fi

if output="$(.codex/scripts/task-registry verify-mutation-hook --format "$format" 2>&1)"; then
  emit_json allow
else
  emit_deny "mutation gate failed: ${output}"
  exit 0
fi
