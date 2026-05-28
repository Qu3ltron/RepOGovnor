#!/usr/bin/env bash
# Install portable agent-governance templates into a project workspace.
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG=""
TARGET_ROOT=""
DRY_RUN=0
FORCE=0
OVERLAY=0

usage() {
  cat <<'EOF'
Usage: install-to-workspace.sh [OPTIONS]

Render templates from project.config.toml into the target repo.

Options:
  --config PATH   Config file (default: ./project.config.toml or plugin example)
  --target PATH   Repo root (default: git toplevel from cwd)
  --overlay       Merge AGENTS.md/GEMINI.md blocks; skip existing infra unless --force
  --dry-run       Show what would run
  --force         Overwrite existing infrastructure files
  -h, --help      Show help

Modes:
  default   Replace AGENTS.md, GEMINI.md, and infrastructure from templates
  --overlay Update <!-- agent-governance:begin/end --> blocks in AGENTS/GEMINI;
            skip .codex, hooks, skills, and tools if they already exist

Steps:
  1. Copy project.config.example.toml to project.config.toml and edit
  2. Run this script from the target repo or pass --target
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --config) CONFIG="$2"; shift 2 ;;
    --target) TARGET_ROOT="$2"; shift 2 ;;
    --overlay) OVERLAY=1; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

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

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "Would render using:"
  echo "  config:   ${CONFIG}"
  echo "  target:   ${TARGET_ROOT}"
  echo "  plugin:   ${PLUGIN_ROOT}"
  echo "  overlay:  ${OVERLAY}"
  echo "  force:    ${FORCE}"
  exit 0
fi

export OVERLAY FORCE
chmod +x "${PLUGIN_ROOT}/scripts/render-from-config.sh"
"${PLUGIN_ROOT}/scripts/render-from-config.sh" "$CONFIG" "$TARGET_ROOT"

echo ""
echo "Required for CI (commit these — plugins/agent-governance/REQUIREMENTS.toml):"
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
echo "Posture check (must pass before push):"
echo "  ${PLUGIN_ROOT}/scripts/status.sh --strict"
echo "  cargo run --bin task_registry -- validate"
echo ""
echo "Optional if plugin link was missing (install creates it when absent):"
echo "  ls -la ${TARGET_ROOT}/.agents/plugins/agent-governance"
