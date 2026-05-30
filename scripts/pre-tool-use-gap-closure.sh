#!/usr/bin/env bash
# Delegates to the workspace canonical mutation gate script.
set -euo pipefail
root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
exec "${root}/tools/agent-governance/pre-tool-use-gap-closure.sh" "$@"
