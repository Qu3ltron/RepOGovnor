#!/usr/bin/env bash
# Validate installer mode semantics against a real temporary git workspace.
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_ROOT="$(mktemp -d)"
OUT_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TARGET_ROOT" "$OUT_DIR"
}
trap cleanup EXIT

reject_grep() {
  local needle="$1"
  local file="$2"
  if grep -q "$needle" "$file"; then
    echo "unexpected match in $file: $needle" >&2
    exit 1
  fi
}

git init -q "$TARGET_ROOT"
mkdir -p "$TARGET_ROOT/.codex/hooks" "$TARGET_ROOT/.agents/plugins" "$TARGET_ROOT/tools/antigravity" "$TARGET_ROOT/plugins" "$TARGET_ROOT/.gemini"
mkdir -p "$TARGET_ROOT/.agents/skills" "$TARGET_ROOT/.cursor/skills" "$TARGET_ROOT/.claude/skills"
git clone -q "$PLUGIN_ROOT" "$TARGET_ROOT/plugins/agent-governance"

printf 'custom agents\n' > "$TARGET_ROOT/AGENTS.md"
printf 'custom gemini\n<!-- agent-governance:begin -->\nold\n<!-- agent-governance:end -->\n' > "$TARGET_ROOT/GEMINI.md"
printf 'old config\n' > "$TARGET_ROOT/.codex/config.toml"
printf 'stale\n' > "$TARGET_ROOT/.codex/settings.toml"
printf 'stale plan approval hook\n' > "$TARGET_ROOT/.codex/hooks/user-plan-approval.toml"
printf 'stale root hook\n' > "$TARGET_ROOT/hooks.json"
printf 'stale gemini settings\n' > "$TARGET_ROOT/.gemini/settings.json"
printf 'stale agy hook\n' > "$TARGET_ROOT/tools/antigravity/pre-tool-use-gap-closure.sh"
printf 'wrong plugin link occupant\n' > "$TARGET_ROOT/.agents/plugins/agent-governance"

for skill in gap-closure-contract task-registry-flow; do
  mkdir -p "$TARGET_ROOT/.cursor/skills/$skill"
  printf 'legacy cursor skill\n' > "$TARGET_ROOT/.cursor/skills/$skill/SKILL.md"
  printf 'legacy symlink target project\n' > "$TARGET_ROOT/.cursor/skills/$skill/PROJECT.md"
  ln -s "../../.cursor/skills/$skill" "$TARGET_ROOT/.agents/skills/$skill"
  mkdir -p "$TARGET_ROOT/.claude/skills/$skill"
  printf 'legacy claude skill\n' > "$TARGET_ROOT/.claude/skills/$skill/SKILL.md"
done

hash_workspace() {
  (
    cd "$TARGET_ROOT"
    {
      find . -type f -print0 \
        | sort -z \
        | while IFS= read -r -d '' path; do sha256sum "$path"; done
      find . -type l -print0 \
        | sort -z \
        | while IFS= read -r -d '' path; do
            printf 'symlink  %s -> %s\n' "$path" "$(readlink "$path")"
          done
    } | sha256sum
  )
}

if "${PLUGIN_ROOT}/scripts/install-to-workspace.sh" \
  --config "${PLUGIN_ROOT}/project.config.example.toml" \
  --target "$TARGET_ROOT" >"$OUT_DIR/no-mode.out" 2>"$OUT_DIR/no-mode.err"; then
  echo "install without a mode unexpectedly succeeded" >&2
  exit 1
fi
grep -q 'missing install mode' "$OUT_DIR/no-mode.err"

if "${PLUGIN_ROOT}/scripts/install-to-workspace.sh" \
  --config "${PLUGIN_ROOT}/project.config.example.toml" \
  --target "$TARGET_ROOT" \
  --overlay >"$OUT_DIR/overlay.out" 2>"$OUT_DIR/overlay.err"; then
  echo "--overlay unexpectedly succeeded" >&2
  exit 1
fi
grep -q -- '--overlay has been removed' "$OUT_DIR/overlay.err"

unsafe_config="$OUT_DIR/unsafe-hook-path.toml"
sed 's#hook_script_path = "tools/agent-governance/pre-tool-use-gap-closure.sh"#hook_script_path = "../outside.sh"#' \
  "$PLUGIN_ROOT/project.config.example.toml" > "$unsafe_config"
if MODE=merge "$PLUGIN_ROOT/scripts/render-from-config.sh" \
  "$unsafe_config" "$TARGET_ROOT" >"$OUT_DIR/unsafe-hook-path.out" 2>"$OUT_DIR/unsafe-hook-path.err"; then
  echo "unsafe hook_script_path unexpectedly succeeded" >&2
  exit 1
fi
grep -q 'unsafe project.config.toml' "$OUT_DIR/unsafe-hook-path.err"

before_sha="$(hash_workspace)"
"${PLUGIN_ROOT}/scripts/install-to-workspace.sh" \
  --config "${PLUGIN_ROOT}/project.config.example.toml" \
  --target "$TARGET_ROOT" \
  --dry-run > "$OUT_DIR/dry-run.out"
after_sha="$(hash_workspace)"

[[ "$before_sha" == "$after_sha" ]]
grep -q 'would-update' "$OUT_DIR/dry-run.out"
grep -q 'AGENTS.md: would-update' "$OUT_DIR/dry-run.out"
grep -q 'would-remove-stale' "$OUT_DIR/dry-run.out"
grep -q '.agents/skills/gap-closure-contract: would-replace-symlink' "$OUT_DIR/dry-run.out"
grep -q '.agents/skills/task-registry-flow: would-replace-symlink' "$OUT_DIR/dry-run.out"
test -L "$TARGET_ROOT/.agents/skills/gap-closure-contract"
test -L "$TARGET_ROOT/.agents/skills/task-registry-flow"
grep -q '.claude/settings.json: would-create' "$OUT_DIR/dry-run.out"
grep -q 'Dry run only; no files changed.' "$OUT_DIR/dry-run.out"
grep -q 'rerun with --merge for an existing repo, or --force for an intentional rebaseline' "$OUT_DIR/dry-run.out"

if (cd "$TARGET_ROOT" && "$PLUGIN_ROOT/scripts/status.sh" --strict > "$OUT_DIR/no-marker-status.out" 2>&1); then
  echo "strict status unexpectedly accepted markerless AGENTS.md" >&2
  exit 1
fi
grep -q 'AGENTS.md missing governance marker block' "$OUT_DIR/no-marker-status.out"

BAD_CARGO_TARGET="$OUT_DIR/unwritable-cargo-target"
mkdir -p "$BAD_CARGO_TARGET"
chmod 0555 "$BAD_CARGO_TARGET"
if (cd "$TARGET_ROOT" && CARGO_TARGET_DIR="$BAD_CARGO_TARGET" "$PLUGIN_ROOT/scripts/status.sh" --strict > "$OUT_DIR/bad-cargo-target-status.out" 2>&1); then
  echo "strict status unexpectedly accepted markerless AGENTS.md with inherited bad CARGO_TARGET_DIR" >&2
  exit 1
fi
chmod 0755 "$BAD_CARGO_TARGET"
grep -q 'AGENTS.md missing governance marker block' "$OUT_DIR/bad-cargo-target-status.out"
reject_grep 'missing status diagnostic' "$OUT_DIR/bad-cargo-target-status.out"

printf 'mentions agent-governance:begin and agent-governance:end in prose\n' > "$TARGET_ROOT/AGENTS.md"
printf '<!-- agent-governance:end -->\nold\n<!-- agent-governance:begin -->\n' > "$TARGET_ROOT/GEMINI.md"
if (cd "$TARGET_ROOT" && "$PLUGIN_ROOT/scripts/status.sh" --strict > "$OUT_DIR/malformed-marker-status.out" 2>&1); then
  echo "strict status unexpectedly accepted malformed marker blocks" >&2
  exit 1
fi
grep -q 'AGENTS.md missing governance marker block' "$OUT_DIR/malformed-marker-status.out"
grep -q 'GEMINI.md governance markers malformed (marker block out of order)' "$OUT_DIR/malformed-marker-status.out"

printf '<!-- agent-governance:begin -->\nstale managed block\n<!-- agent-governance:end -->\n' > "$TARGET_ROOT/AGENTS.md"
printf '<!-- agent-governance:begin -->\nstale managed block\n<!-- agent-governance:end -->\n' > "$TARGET_ROOT/GEMINI.md"
if (cd "$TARGET_ROOT" && "$PLUGIN_ROOT/scripts/status.sh" --strict > "$OUT_DIR/stale-marker-status.out" 2>&1); then
  echo "strict status unexpectedly accepted stale marker blocks" >&2
  exit 1
fi
grep -q 'AGENTS.md governance markers malformed (stale marker content)' "$OUT_DIR/stale-marker-status.out"
grep -q 'GEMINI.md governance markers malformed (stale marker content)' "$OUT_DIR/stale-marker-status.out"

printf 'custom agents\n' > "$TARGET_ROOT/AGENTS.md"
printf 'custom gemini\n<!-- agent-governance:begin -->\nold\n<!-- agent-governance:end -->\n' > "$TARGET_ROOT/GEMINI.md"

"${PLUGIN_ROOT}/scripts/install-to-workspace.sh" \
  --config "${PLUGIN_ROOT}/project.config.example.toml" \
  --target "$TARGET_ROOT" \
  --merge > "$OUT_DIR/merge.out"

grep -q 'custom agents' "$TARGET_ROOT/AGENTS.md"
grep -q 'agent-governance:begin' "$TARGET_ROOT/AGENTS.md"
grep -q 'First-run next steps:' "$OUT_DIR/merge.out"
grep -q 'docs/example-workflow.md' "$OUT_DIR/merge.out"
grep -q 'docs/migration-v2.md' "$OUT_DIR/merge.out"
grep -q '.codex/scripts/task-registry verify-chain --format json' "$OUT_DIR/merge.out"
test ! -e "$TARGET_ROOT/.codex/settings.toml"
test ! -e "$TARGET_ROOT/.codex/hooks/user-plan-approval.toml"
test ! -e "$TARGET_ROOT/hooks.json"
test ! -e "$TARGET_ROOT/.gemini/settings.json"
test ! -e "$TARGET_ROOT/tools/antigravity/pre-tool-use-gap-closure.sh"
grep -q 'remove-stale' "$OUT_DIR/merge.out"
reject_grep 'preserve-stale' "$OUT_DIR/merge.out"
grep -q 'preserve-drift' "$OUT_DIR/merge.out"
grep -q 'hooks = true' "$TARGET_ROOT/.codex/config.toml"
grep -q '.agents/skills/gap-closure-contract: replace-symlink' "$OUT_DIR/merge.out"
grep -q '.agents/skills/task-registry-flow: replace-symlink' "$OUT_DIR/merge.out"
reject_grep '.agents/skills/gap-closure-contract: aligned' "$OUT_DIR/merge.out"
reject_grep '.agents/skills/task-registry-flow: aligned' "$OUT_DIR/merge.out"
reject_grep '.agents/skills/gap-closure-contract: preserve-drift' "$OUT_DIR/merge.out"
reject_grep '.agents/skills/task-registry-flow: preserve-drift' "$OUT_DIR/merge.out"
test ! -L "$TARGET_ROOT/.agents/skills/gap-closure-contract"
test ! -L "$TARGET_ROOT/.agents/skills/task-registry-flow"
test -d "$TARGET_ROOT/.agents/skills/gap-closure-contract"
test -d "$TARGET_ROOT/.agents/skills/task-registry-flow"
grep -q 'name: gap-closure-contract' "$TARGET_ROOT/.agents/skills/gap-closure-contract/SKILL.md"
grep -q 'name: task-registry-flow' "$TARGET_ROOT/.agents/skills/task-registry-flow/SKILL.md"
reject_grep 'legacy symlink target project' "$TARGET_ROOT/.agents/skills/gap-closure-contract/PROJECT.md"
reject_grep 'legacy symlink target project' "$TARGET_ROOT/.agents/skills/task-registry-flow/PROJECT.md"
test -f "$TARGET_ROOT/docs/task-registry.toml"
mkdir -p "$TARGET_ROOT/nested/work"
(
  cd "$TARGET_ROOT/nested/work"
  "$TARGET_ROOT/.codex/scripts/task-registry" validate > "$OUT_DIR/nested-validate.out"
)
grep -q 'task registry validate ok' "$OUT_DIR/nested-validate.out"
test -f "$TARGET_ROOT/docs/task-registry/events.jsonl"
test ! -e "$TARGET_ROOT/nested/work/docs/task-registry.toml"
test ! -e "$TARGET_ROOT/nested/work/docs/task-registry/events.jsonl"

cat > "$OUT_DIR/failing-verify-hook.sh" <<'EOF'
#!/usr/bin/env bash
printf 'quote " slash \\ tab \t cr \r newline\nnext'
exit 1
EOF
chmod +x "$OUT_DIR/failing-verify-hook.sh"
printf 'GOVERNANCE_VERIFY_HOOK_CMD="%s"\n' "$OUT_DIR/failing-verify-hook.sh" > "$TARGET_ROOT/.codex/governance-cli.env"
for format in codex claude cursor antigravity; do
  (
    cd "$TARGET_ROOT"
    tools/agent-governance/pre-tool-use-gap-closure.sh --format "$format" \
      > "$OUT_DIR/hook-deny-$format.json"
  )
  python3 - "$format" "$OUT_DIR/hook-deny-$format.json" <<'PY'
import json
import sys
fmt = sys.argv[1]
payload = json.loads(open(sys.argv[2], encoding="utf-8").read())
if fmt in {"codex", "claude"}:
    reason = payload["hookSpecificOutput"]["permissionDecisionReason"]
elif fmt == "cursor":
    reason = payload["user_message"]
else:
    reason = payload["reason"]
assert "noncanonical GOVERNANCE_VERIFY_HOOK_CMD" in reason
PY
done
printf 'GOVERNANCE_VERIFY_HOOK_CMD=".codex/scripts/task-registry verify-mutation-hook"\n' > "$TARGET_ROOT/.codex/governance-cli.env"

# Claude Code merge checks
test -f "$TARGET_ROOT/.claude/settings.json"
grep -q 'GOVERNANCE_HOOK_FORMAT=claude' "$TARGET_ROOT/.claude/settings.json"
grep -q 'GOVERNANCE_VERIFY_HOOK_CMD' "$TARGET_ROOT/.claude/settings.json"
grep -q 'PreToolUse' "$TARGET_ROOT/.claude/settings.json"
grep -q '"matcher": "Bash|Edit|Write"' "$TARGET_ROOT/.claude/settings.json"
test -d "$TARGET_ROOT/.claude/skills/gap-closure-contract"
test -d "$TARGET_ROOT/.claude/skills/task-registry-flow"
grep -q 'name: gap-closure-contract' "$TARGET_ROOT/.claude/skills/gap-closure-contract/SKILL.md"
grep -q 'name: task-registry-flow' "$TARGET_ROOT/.claude/skills/task-registry-flow/SKILL.md"

# Explicit Claude Code environment check (tolerate tracked_for_ci failures in test workspace)
(cd "$TARGET_ROOT" && "$PLUGIN_ROOT/scripts/status.sh" --strict --env claude > "$OUT_DIR/claude-status.out" 2>&1) || true
grep -q 'claude CLI' "$OUT_DIR/claude-status.out"
grep -q '.claude/settings.json is valid JSON' "$OUT_DIR/claude-status.out"
grep -q '.claude/settings.json PreToolUse hook delegates to canonical gate with claude format' "$OUT_DIR/claude-status.out"

for skill in gap-closure-contract task-registry-flow; do
  rm -rf "$TARGET_ROOT/.agents/skills/$skill"
  ln -s "../../.cursor/skills/$skill" "$TARGET_ROOT/.agents/skills/$skill"
done
printf 'stale\n' > "$TARGET_ROOT/.codex/settings.toml"
printf 'stale plan approval hook\n' > "$TARGET_ROOT/.codex/hooks/user-plan-approval.toml"
printf 'stale root hook\n' > "$TARGET_ROOT/hooks.json"
printf 'stale gemini settings\n' > "$TARGET_ROOT/.gemini/settings.json"
printf 'stale agy hook\n' > "$TARGET_ROOT/tools/antigravity/pre-tool-use-gap-closure.sh"

if (cd "$TARGET_ROOT" && "$PLUGIN_ROOT/scripts/status.sh" --strict > "$OUT_DIR/symlink-status.out" 2>&1); then
  echo "strict status unexpectedly accepted symlinked skill projections" >&2
  exit 1
fi
grep -q '.agents/skills/gap-closure-contract must be a native directory, not a symlink' "$OUT_DIR/symlink-status.out"
grep -q '.agents/skills/task-registry-flow must be a native directory, not a symlink' "$OUT_DIR/symlink-status.out"

"${PLUGIN_ROOT}/scripts/install-to-workspace.sh" \
  --config "${PLUGIN_ROOT}/project.config.example.toml" \
  --target "$TARGET_ROOT" \
  --force > "$OUT_DIR/force.out"

reject_grep 'custom agents' "$TARGET_ROOT/AGENTS.md"
grep -q 'First-run next steps:' "$OUT_DIR/force.out"
grep -q 'Posture checks:' "$OUT_DIR/force.out"
grep -q '.codex/scripts/task-registry validate' "$OUT_DIR/force.out"
test ! -e "$TARGET_ROOT/.codex/settings.toml"
test ! -e "$TARGET_ROOT/.codex/hooks/user-plan-approval.toml"
test ! -e "$TARGET_ROOT/hooks.json"
test ! -e "$TARGET_ROOT/.gemini/settings.json"
test ! -e "$TARGET_ROOT/tools/antigravity/pre-tool-use-gap-closure.sh"
test -L "$TARGET_ROOT/.agents/plugins/agent-governance"
[[ "$(readlink "$TARGET_ROOT/.agents/plugins/agent-governance")" == "../../plugins/agent-governance" ]]
grep -q 'agent-governance:begin' "$TARGET_ROOT/AGENTS.md"
grep -q 'agent-governance:begin' "$TARGET_ROOT/GEMINI.md"
grep -q '.agents/skills/gap-closure-contract: replace-symlink' "$OUT_DIR/force.out"
grep -q '.agents/skills/task-registry-flow: replace-symlink' "$OUT_DIR/force.out"
reject_grep '.agents/skills/gap-closure-contract: aligned' "$OUT_DIR/force.out"
reject_grep '.agents/skills/task-registry-flow: aligned' "$OUT_DIR/force.out"
reject_grep '.agents/skills/gap-closure-contract: preserve-drift' "$OUT_DIR/force.out"
reject_grep '.agents/skills/task-registry-flow: preserve-drift' "$OUT_DIR/force.out"
test ! -L "$TARGET_ROOT/.agents/skills/gap-closure-contract"
test ! -L "$TARGET_ROOT/.agents/skills/task-registry-flow"
test -d "$TARGET_ROOT/.agents/skills/gap-closure-contract"
test -d "$TARGET_ROOT/.agents/skills/task-registry-flow"

# Claude Code force checks
test -f "$TARGET_ROOT/.claude/settings.json"
grep -q 'GOVERNANCE_HOOK_FORMAT=claude' "$TARGET_ROOT/.claude/settings.json"
test -d "$TARGET_ROOT/.claude/skills/gap-closure-contract"
test -d "$TARGET_ROOT/.claude/skills/task-registry-flow"

echo "install mode validation ok"
