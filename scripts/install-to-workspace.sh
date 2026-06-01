#!/usr/bin/env bash
# Install portable agent-governance templates into a project workspace.
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG=""
TARGET_ROOT=""
MODE=""
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage: install-to-workspace.sh [OPTIONS]

Render templates from project.config.toml into the target repo.

Options:
  --config PATH   Config file (default: ./project.config.toml or plugin example)
  --target PATH   Repo root (default: git toplevel from cwd)
  --dry-run       Project the complete force install without writing files
  --merge         Add missing files and update managed governance surfaces
  --force         Apply the dry-run projection: overwrite managed files and remove stale paths
  -h, --help      Show help

Modes:
  --dry-run  No mutation; prints the complete create/update/remove/chmod projection.
  --merge    Existing repo path; merges AGENTS/GEMINI markers, refreshes managed
             plugin files, preserves project registry/events and stale paths.
  --force    Rebaseline path; matches --dry-run, including stale path removal.

Steps:
  1. Copy project.config.example.toml to project.config.toml and edit
  2. Run this script from the target repo or pass --target
EOF
}

set_mode() {
  local requested="$1"
  if [[ -n "$MODE" ]]; then
    echo "choose exactly one install mode: --dry-run, --merge, or --force" >&2
    exit 1
  fi
  MODE="$requested"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --config) CONFIG="$2"; shift 2 ;;
    --target) TARGET_ROOT="$2"; shift 2 ;;
    --dry-run) set_mode force; DRY_RUN=1; shift ;;
    --merge) set_mode merge; shift ;;
    --force) set_mode force; shift ;;
    --overlay)
      echo "--overlay has been removed. Use --merge for existing repos or --force for rebaseline." >&2
      exit 1
      ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

if [[ -z "$MODE" ]]; then
  echo "missing install mode: choose --dry-run, --merge, or --force" >&2
  usage >&2
  exit 1
fi

if [[ -z "$TARGET_ROOT" ]]; then
  TARGET_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi
TARGET_ROOT="$(cd "$TARGET_ROOT" && pwd)"

if [[ -z "$CONFIG" ]]; then
  if [[ -f "${TARGET_ROOT}/project.config.toml" ]]; then
    CONFIG="${TARGET_ROOT}/project.config.toml"
  elif [[ -f "${PLUGIN_ROOT}/project.config.toml" ]]; then
    CONFIG="${PLUGIN_ROOT}/project.config.toml"
  else
    echo "No project.config.toml found. Copy and edit:" >&2
    echo "  cp ${PLUGIN_ROOT}/project.config.example.toml ${TARGET_ROOT}/project.config.toml" >&2
    echo "  # or for Spectrum Arcana reference:" >&2
    echo "  --config ${PLUGIN_ROOT}/examples/spectrum-arcana.project.config.toml" >&2
    exit 1
  fi
fi

export MODE DRY_RUN
bash "${PLUGIN_ROOT}/scripts/render-from-config.sh" "$CONFIG" "$TARGET_ROOT"

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo ""
  echo "Dry run only; no files changed."
  echo "Next step:"
  echo "  rerun with --merge for an existing repo, or --force for an intentional rebaseline"
  exit 0
fi

echo ""
echo "First-run next steps:"
echo "  1. Review the rendered action list above."
echo "  2. Run the posture checks below before committing or pushing."
echo "  3. Read ${PLUGIN_ROOT}/docs/example-workflow.md for the plan -> activate -> land loop."
echo "  4. Read ${PLUGIN_ROOT}/docs/migration-v2.md when upgrading an existing repo."
echo ""
echo "Required for CI (commit these; source: plugins/agent-governance/REQUIREMENTS.toml):"
python3 <<PY
import tomllib
from pathlib import Path
req = tomllib.loads(Path("${PLUGIN_ROOT}/REQUIREMENTS.toml").read_text(encoding="utf-8"))
for path in req["tracked_for_ci"]["required"]:
    print(f"  git add {path}")
print()
print(req["tracked_for_ci"]["design_rule"].strip())
PY
echo ""
echo "Posture checks:"
echo "  ${PLUGIN_ROOT}/scripts/status.sh --strict"
echo "  .codex/scripts/task-registry validate"
echo "  .codex/scripts/task-registry source-limit check"
echo "  .codex/scripts/task-registry verify-chain --format json"
echo "  .codex/scripts/task-registry metrics"
echo ""
echo "Optional if plugin link was missing (install creates it when absent):"
echo "  ls -la ${TARGET_ROOT}/.agents/plugins/agent-governance"
