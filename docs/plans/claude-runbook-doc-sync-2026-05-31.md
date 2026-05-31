# Claude Runbook Documentation Sync Gap Closure Contract

## Approved Scope

Close the remaining agent-facing documentation drift in the Claude Code
`run-governance-plugin` skill and its smoke-test labels.

In scope:

1. Remove stale hardcoded test/subcommand counts and outdated failure notes from
   the Claude runbook.
2. Keep the smoke driver labels count-free so future test additions do not make
   the runbook stale.

Out of scope: runtime CLI behavior, smoke-test coverage expansion, release
version changes, compatibility shims, and edits outside
`/home/hasnamuss/reclaimed/work/Governance-plugin`.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/claude-runbook-doc-sync-2026-05-31.md` - `closure_contract`: activate before runbook edits; acceptance proof: `.codex/scripts/task-registry activate docs/plans/claude-runbook-doc-sync-2026-05-31.md`.
- [ ] `[VERIFY]` `.codex/agent-governance.toml` - `workspace_boundary`: confirm repo root; acceptance proof: `git rev-parse --show-toplevel`.

### Phase 1: Runbook and smoke labels

- [ ] `[MODIFY]` `.claude/skills/run-governance-plugin/SKILL.md` - `runbook_current_gates`: document current count-free smoke coverage and strict status behavior.
- [ ] `[MODIFY]` `.claude/skills/run-governance-plugin/smoke.sh` - `count_free_labels`: remove stale hardcoded test-count label and stale known-nonzero status wording.

### Phase 2: Validation and handoff

- [ ] `[VERIFY]` `.claude/skills/run-governance-plugin/SKILL.md` - `runbook_positive`: current gates are documented.
- [ ] `[VERIFY]` `.claude/skills/run-governance-plugin/SKILL.md` - `runbook_negative`: stale hardcoded claims are absent.
- [ ] `[VERIFY]` `.claude/skills/run-governance-plugin/smoke.sh` - `smoke_syntax`: `bash -n .claude/skills/run-governance-plugin/smoke.sh`.
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `source_limit`: `.codex/scripts/task-registry source-limit check`.

## Per-Gap Success Criteria

### GAP-CRD01: Claude runbook no longer carries stale smoke-test claims

- Current failure: `.claude/skills/run-governance-plugin/SKILL.md` and its
  smoke script still reference old test/subcommand counts and an old strict
  status warning posture.
- Good behavior: Given the Claude runbook, when an agent reads the smoke-test
  path, then it sees count-free coverage, current `status.sh --strict`
  expectations, and release-source/receipt-chain checks.
- Forbidden behavior: The runbook and smoke script must not contain `95 tests`,
  `All 14 CLI subcommands`, `untracked .claude`, or hardcoded stale wrapper
  path claims.
- Files involved: `.claude/skills/run-governance-plugin/SKILL.md`,
  `.claude/skills/run-governance-plugin/smoke.sh`.
- Positive test: `bash -c 'rg -q "cargo test passes without hardcoded count assumptions" .claude/skills/run-governance-plugin/SKILL.md && rg -q "release-check all --format json" .claude/skills/run-governance-plugin/SKILL.md && rg -q "receipt chain" .claude/skills/run-governance-plugin/SKILL.md'`.
- Negative test: `bash -c '! rg -n "95 tests|All 14 CLI subcommands|untracked \\.claude|hardcoded development path" .claude/skills/run-governance-plugin/SKILL.md .claude/skills/run-governance-plugin/smoke.sh'`.
- Data/schema/provenance: No schema change; runbook-only behavior.
- Runtime: Smoke script remains syntactically valid and still runs the same
  command set.

## Validation Plan

Focused:

- `bash -c 'rg -q "cargo test passes without hardcoded count assumptions" .claude/skills/run-governance-plugin/SKILL.md && rg -q "release-check all --format json" .claude/skills/run-governance-plugin/SKILL.md && rg -q "receipt chain" .claude/skills/run-governance-plugin/SKILL.md'`
- `bash -c '! rg -n "95 tests|All 14 CLI subcommands|untracked \\.claude|hardcoded development path" .claude/skills/run-governance-plugin/SKILL.md .claude/skills/run-governance-plugin/smoke.sh'`
- `bash -n .claude/skills/run-governance-plugin/smoke.sh`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `scripts/status.sh --strict`

## Walkthrough Evidence

Capture after implementation:

- `TASK_REPORT PLAN-2026-05-31-claude-runbook-doc-sync`: all tasks completed,
  no deferred or blocked tasks.
- `TASK_METRICS`: `active=0`, `deferred=0`, `blocked=0`,
  `receipt_chain_breaks=0`, and `unchained_events=0`.
- Positive and negative runbook checks exit zero.
- `bash -n .claude/skills/run-governance-plugin/smoke.sh` exits zero.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-claude-runbook-doc-sync"

[[behaviors]]
behavior_id = "B-CRD01-positive"
gap_id = "GAP-CRD01"
polarity = "positive"
title = "Claude runbook states current count-free smoke coverage"
given = "The Claude runbook exists"
when = "The current smoke-test contract is searched"
then = "The runbook documents count-free cargo test behavior, release-check JSON, and receipt chain checks"
confirmation = "bash -c 'rg -q \"cargo test passes without hardcoded count assumptions\" .claude/skills/run-governance-plugin/SKILL.md && rg -q \"release-check all --format json\" .claude/skills/run-governance-plugin/SKILL.md && rg -q \"receipt chain\" .claude/skills/run-governance-plugin/SKILL.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'rg -q \"cargo test passes without hardcoded count assumptions\" .claude/skills/run-governance-plugin/SKILL.md && rg -q \"release-check all --format json\" .claude/skills/run-governance-plugin/SKILL.md && rg -q \"receipt chain\" .claude/skills/run-governance-plugin/SKILL.md'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-CRD01-negative"
gap_id = "GAP-CRD01"
polarity = "negative"
title = "Claude runbook has no stale smoke-count claims"
given = "The Claude runbook and smoke script exist"
when = "Stale smoke-test claims are searched"
then = "No hardcoded old count or obsolete strict-status warning remains"
confirmation = "bash -c '! rg -n \"95 tests|All 14 CLI subcommands|untracked \\\\.claude|hardcoded development path\" .claude/skills/run-governance-plugin/SKILL.md .claude/skills/run-governance-plugin/smoke.sh'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! rg -n \"95 tests|All 14 CLI subcommands|untracked \\\\.claude|hardcoded development path\" .claude/skills/run-governance-plugin/SKILL.md .claude/skills/run-governance-plugin/smoke.sh'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-CRD01-validation"
gap_id = "GAP-CRD01"
polarity = "validation"
title = "Claude smoke script remains valid shell"
given = "The smoke script exists"
when = "Bash parses it without executing commands"
then = "Shell syntax is valid"
confirmation = "bash -n .claude/skills/run-governance-plugin/smoke.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -n .claude/skills/run-governance-plugin/smoke.sh"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-CRD01"
status = "planned"
kind = "governance"
reason = "Claude-facing runbook documentation must not lag current release gates."
behavior_ids = ["B-CRD01-positive", "B-CRD01-negative", "B-CRD01-validation"]
title = "Refresh Claude runbook smoke documentation"
acceptance_proof = "Behaviors B-CRD01-positive, B-CRD01-negative, and B-CRD01-validation pass."

[[tasks.targets]]
file = ".claude/skills/run-governance-plugin/SKILL.md"
object = "runbook_current_gates"
required_change = "Document current count-free smoke coverage and strict status behavior."

[[tasks.targets]]
file = ".claude/skills/run-governance-plugin/smoke.sh"
object = "count_free_labels"
required_change = "Remove stale hardcoded count labels and obsolete known-nonzero status wording."
```
