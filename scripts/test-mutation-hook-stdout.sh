#!/usr/bin/env bash
# Assert the canonical mutation gate emits exactly one JSON object on stdout.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

hook="${MUTATION_HOOK_SCRIPT:-tools/agent-governance/pre-tool-use-gap-closure.sh}"
if [[ ! -f "$hook" ]]; then
  echo "FAIL: mutation hook not found: $hook" >&2
  exit 1
fi

out="$(printf '{}' | GOVERNANCE_HOOK_FORMAT=cursor bash "$hook" --format cursor)"
if printf '%s' "$out" | grep -q '^TASK_VERIFY'; then
  echo "FAIL: verify-mutation-hook leaked to stdout: $out" >&2
  exit 1
fi
printf '%s' "$out" | python3 -c 'import json,sys; json.load(sys.stdin)'
echo "ok: single valid JSON on hook stdout"
