# Documentation Sync Gap Closure Contract

## Approved Scope

Update the user-facing README, agent-facing instructions, generated templates,
skill instructions, and system docs so the documented production contract
matches the current hardened runtime.

In scope:

1. README and system docs must state the current production release contract:
   clean tree, manifest-backed release source, native required files, receipt
   chain integrity, terminal task immutability, and Nix package/module checks.
2. Agent-facing docs must tell Codex, Antigravity, Cursor, and Claude Code the
   same operational rules: exact targets, fail-closed mutation parsing,
   no terminal task rewrites, no final-release waivers, and no compatibility
   shims.
3. Templates and skill projections must match the canonical skill source so new
   installs receive the same instructions.

Out of scope: runtime behavior changes, public CLI changes, release version
changes, compatibility shims, and edits outside
`/home/hasnamuss/reclaimed/work/Governance-plugin`.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/documentation-sync-2026-05-31.md` - `closure_contract`: activate this contract before documentation edits; acceptance proof: `.codex/scripts/task-registry activate docs/plans/documentation-sync-2026-05-31.md`.
- [ ] `[VERIFY]` `.codex/agent-governance.toml` - `workspace_boundary`: confirm `git rev-parse --show-toplevel` matches the configured repo root; acceptance proof: `git rev-parse --show-toplevel`.

### Phase 1: README and system documentation

- [ ] `[MODIFY]` `README.md` - `production_contract`: document clean-tree release checks, manifest-backed release source, native files, receipt chain integrity, terminal task immutability, and Nix verification.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `runtime_invariants`: add agent-facing runtime invariants for receipts, mutation target parsing, terminal tasks, release source, and final release waivers.
- [ ] `[MODIFY]` `docs/releases/v2.md` - `release_checklist`: replace stale artifact enumeration with `REQUIREMENTS.toml` authority and add current final gates.
- [ ] `[MODIFY]` `docs/agent-environment-matrix.md` - `environment_matrix`: include Claude Code and the shared hardening rules all agents must follow.

### Phase 2: Agent-facing docs and templates

- [ ] `[MODIFY]` `AGENTS.md` - `agent_hardening_rules`: add shared production hardening rules for agents.
- [ ] `[MODIFY]` `CLAUDE.md` - `agent_hardening_rules`: keep Claude Code instructions aligned with `AGENTS.md`.
- [ ] `[MODIFY]` `GEMINI.md` - `agent_hardening_rules`: keep Antigravity instructions aligned with `AGENTS.md`.
- [ ] `[MODIFY]` `.cursor/rules/agent-governance.mdc` - `cursor_hardening_rules`: state Cursor-specific enforcement expectations.
- [ ] `[MODIFY]` `templates/AGENTS.md.template` - `agent_hardening_rules_template`: render the same rules for future Codex installs.
- [ ] `[MODIFY]` `templates/CLAUDE.md.template` - `agent_hardening_rules_template`: render the same rules for future Claude Code installs.
- [ ] `[MODIFY]` `templates/GEMINI.md.template` - `agent_hardening_rules_template`: render the same rules for future Antigravity installs.
- [ ] `[MODIFY]` `templates/.cursor/rules/agent-governance.mdc.template` - `cursor_hardening_rules_template`: render Cursor rules for future installs.

### Phase 3: Skill source and projections

- [ ] `[MODIFY]` `skills/gap-closure-contract/SKILL.md` - `contract_requirements`: include documentation sync, terminal task, receipt-chain, and release-source requirements.
- [ ] `[MODIFY]` `skills/task-registry-flow/SKILL.md` - `registry_flow_requirements`: include terminal immutability and receipt-chain handoff requirements.
- [ ] `[GENERATE]` `.agents/skills/gap-closure-contract/SKILL.md` - `codex_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.agents/skills/gap-closure-contract.md` - `antigravity_markdown_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.cursor/skills/gap-closure-contract/SKILL.md` - `cursor_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.claude/skills/gap-closure-contract/SKILL.md` - `claude_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.agents/skills/task-registry-flow/SKILL.md` - `codex_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.agents/skills/task-registry-flow.md` - `antigravity_markdown_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.cursor/skills/task-registry-flow/SKILL.md` - `cursor_projection`: mirror canonical skill source.
- [ ] `[GENERATE]` `.claude/skills/task-registry-flow/SKILL.md` - `claude_projection`: mirror canonical skill source.

### Phase 4: Validation and handoff

- [ ] `[VERIFY]` `README.md` - `docs_positive_behavior`: focused documentation contains checks pass.
- [ ] `[VERIFY]` `docs/releases/v2.md` - `docs_negative_behavior`: stale release checklist wording is absent.
- [ ] `[VERIFY]` `skills/gap-closure-contract/SKILL.md` - `projection_alignment`: canonical skill files match every projection.
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `source_limit`: `.codex/scripts/task-registry source-limit check`.
- [ ] `[VERIFY]` `scripts/status.sh` - `strict_posture`: `scripts/status.sh --strict`.

## Per-Gap Success Criteria

### GAP-DOC01: README and system docs describe the hardened production contract

- Current failure: Release-facing docs understate current hardening by omitting
  or scattering clean-tree release checks, native release-source files,
  receipt-chain integrity, terminal task immutability, and Nix verification.
- Good behavior: Given a maintainer preparing a production release, when they
  read `README.md`, `docs/runtime-schemas.md`, `docs/releases/v2.md`, and
  `docs/agent-environment-matrix.md`, then they see the exact gates and
  invariants required for release readiness.
- Forbidden behavior: Docs must not imply final releases may use local waiver
  flags, symlinked required release-source files are acceptable, or
  `docs/releases/v2.md` owns a stale hand-maintained artifact list instead of
  `REQUIREMENTS.toml`.
- Files involved: `README.md`, `docs/runtime-schemas.md`,
  `docs/releases/v2.md`, `docs/agent-environment-matrix.md`.
- Positive test: `bash -c 'rg -q "Production readiness is clean-tree and manifest-backed" README.md && rg -q "native files, not symlinks" README.md docs/runtime-schemas.md docs/releases/v2.md && rg -q "terminal" README.md docs/runtime-schemas.md docs/agent-environment-matrix.md && rg -q "nix flake check --no-build --all-systems" README.md docs/releases/v2.md'`.
- Negative test: `bash -c '! rg -n "Final releases may use local waiver|symlinked required release-source files are acceptable|^## Required Artifacts$" README.md docs/runtime-schemas.md docs/releases/v2.md docs/agent-environment-matrix.md'`.
- Data/schema/provenance: Documentation points to `REQUIREMENTS.toml` as the
  release-source manifest authority; no schema or API changes.
- Runtime: `scripts/status.sh --release-source` remains the clean-tree release
  gate.

### GAP-DOC02: Agent-facing docs and skills reinforce the same canonical rules

- Current failure: Agent-facing docs do not all state the post-hardening
  operational rules, and future install templates can drift from the live repo
  instructions.
- Good behavior: Given any supported agent entry point, when an agent reads its
  instructions or skill docs, then it sees exact task targets, fail-closed
  mutation parsing, terminal task immutability, receipt-chain evidence,
  release-source native-file rules, and zero backward-compatibility posture.
- Forbidden behavior: No agent-facing doc or template should direct agents to
  use project-native registry executors, rewrite completed or cancelled tasks,
  depend on legacy workspace settings, or treat compatibility shims as allowed.
- Files involved: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`,
  `.cursor/rules/agent-governance.mdc`, `templates/AGENTS.md.template`,
  `templates/CLAUDE.md.template`, `templates/GEMINI.md.template`,
  `templates/.cursor/rules/agent-governance.mdc.template`,
  `skills/gap-closure-contract/SKILL.md`,
  `skills/task-registry-flow/SKILL.md`, and their `.agents`, `.cursor`, and
  `.claude` projections.
- Positive test: `bash -c 'for f in AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template; do rg -q "terminal task" "$f"; rg -q "receipt chain" "$f"; done'`.
- Negative test: `bash -c '! rg -n "project-native task_registry|rewrite completed|rewrite cancelled|compatibility shims are allowed|workspace \\.gemini/settings\\.json" AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template skills/gap-closure-contract/SKILL.md skills/task-registry-flow/SKILL.md'`.
- Data/schema/provenance: Canonical skill files and projections must match
  byte-for-byte after generation.
- Runtime: `scripts/status.sh --strict` must confirm skill projection posture.

## Validation Plan

Focused:

- `bash -c 'rg -q "Production readiness is clean-tree and manifest-backed" README.md && rg -q "native files, not symlinks" README.md docs/runtime-schemas.md docs/releases/v2.md && rg -q "terminal" README.md docs/runtime-schemas.md docs/agent-environment-matrix.md && rg -q "nix flake check --no-build --all-systems" README.md docs/releases/v2.md'`
- `bash -c '! rg -n "Final releases may use local waiver|symlinked required release-source files are acceptable|^## Required Artifacts$" README.md docs/runtime-schemas.md docs/releases/v2.md docs/agent-environment-matrix.md'`
- `bash -c 'for f in AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template; do rg -q "terminal task" "$f"; rg -q "receipt chain" "$f"; done'`
- `bash -c '! rg -n "project-native task_registry|rewrite completed|rewrite cancelled|compatibility shims are allowed|workspace \\.gemini/settings\\.json" AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template skills/gap-closure-contract/SKILL.md skills/task-registry-flow/SKILL.md'`
- `bash -c 'cmp -s skills/gap-closure-contract/SKILL.md .agents/skills/gap-closure-contract/SKILL.md && cmp -s skills/gap-closure-contract/SKILL.md .agents/skills/gap-closure-contract.md && cmp -s skills/gap-closure-contract/SKILL.md .cursor/skills/gap-closure-contract/SKILL.md && cmp -s skills/gap-closure-contract/SKILL.md .claude/skills/gap-closure-contract/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md && cmp -s skills/task-registry-flow/SKILL.md .cursor/skills/task-registry-flow/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md'`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry release-check all --format json`
- `scripts/status.sh --strict`
- `scripts/status.sh --release-source`

## Walkthrough Evidence

Capture after implementation:

- `TASK_REPORT PLAN-2026-05-31-documentation-sync`: all tasks completed, no
  deferred or blocked tasks.
- `TASK_METRICS`: `active=0`, `deferred=0`, `blocked=0`,
  `receipt_chain_breaks=0`, and `unchained_events=0`.
- Focused positive and negative documentation checks exit zero.
- Skill source/projection `cmp` checks exit zero.
- Full validation commands exit zero.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-documentation-sync"

[[behaviors]]
behavior_id = "B-DOC01-positive"
gap_id = "GAP-DOC01"
polarity = "positive"
title = "System docs state the hardened production contract"
given = "The README and system docs exist"
when = "The documentation contract is searched"
then = "The current clean-tree, native-file, terminal-task, and Nix release rules are present"
confirmation = "bash -c 'rg -q \"Production readiness is clean-tree and manifest-backed\" README.md && rg -q \"native files, not symlinks\" README.md docs/runtime-schemas.md docs/releases/v2.md && rg -q \"terminal\" README.md docs/runtime-schemas.md docs/agent-environment-matrix.md && rg -q \"nix flake check --no-build --all-systems\" README.md docs/releases/v2.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'rg -q \"Production readiness is clean-tree and manifest-backed\" README.md && rg -q \"native files, not symlinks\" README.md docs/runtime-schemas.md docs/releases/v2.md && rg -q \"terminal\" README.md docs/runtime-schemas.md docs/agent-environment-matrix.md && rg -q \"nix flake check --no-build --all-systems\" README.md docs/releases/v2.md'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-DOC01-negative"
gap_id = "GAP-DOC01"
polarity = "negative"
title = "System docs do not preserve stale release claims"
given = "The README and system docs exist"
when = "Stale release claims are searched"
then = "No final-waiver, symlink-acceptance, or stale required-artifact heading remains"
confirmation = "bash -c '! rg -n \"Final releases may use local waiver|symlinked required release-source files are acceptable|^## Required Artifacts$\" README.md docs/runtime-schemas.md docs/releases/v2.md docs/agent-environment-matrix.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! rg -n \"Final releases may use local waiver|symlinked required release-source files are acceptable|^## Required Artifacts$\" README.md docs/runtime-schemas.md docs/releases/v2.md docs/agent-environment-matrix.md'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-DOC02-positive"
gap_id = "GAP-DOC02"
polarity = "positive"
title = "Agent docs state terminal task and receipt-chain rules"
given = "Live agent docs and install templates exist"
when = "The hardening rules are searched"
then = "Each agent-facing document includes terminal task and receipt chain instructions"
confirmation = "bash -c 'for f in AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template; do rg -q \"terminal task\" \"$f\"; rg -q \"receipt chain\" \"$f\"; done'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'for f in AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template; do rg -q \"terminal task\" \"$f\"; rg -q \"receipt chain\" \"$f\"; done'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-DOC02-negative"
gap_id = "GAP-DOC02"
polarity = "negative"
title = "Agent docs reject noncanonical compatibility paths"
given = "Live agent docs, install templates, and skill docs exist"
when = "Noncanonical compatibility wording is searched"
then = "No instruction permits project-native registry executors, terminal rewrites, compatibility shims, or workspace Gemini settings"
confirmation = "bash -c '! rg -n \"project-native task_registry|rewrite completed|rewrite cancelled|compatibility shims are allowed|workspace \\\\.gemini/settings\\\\.json\" AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template skills/gap-closure-contract/SKILL.md skills/task-registry-flow/SKILL.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! rg -n \"project-native task_registry|rewrite completed|rewrite cancelled|compatibility shims are allowed|workspace \\\\.gemini/settings\\\\.json\" AGENTS.md CLAUDE.md GEMINI.md templates/AGENTS.md.template templates/CLAUDE.md.template templates/GEMINI.md.template .cursor/rules/agent-governance.mdc templates/.cursor/rules/agent-governance.mdc.template skills/gap-closure-contract/SKILL.md skills/task-registry-flow/SKILL.md'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-DOC02-projection"
gap_id = "GAP-DOC02"
polarity = "positive"
title = "Skill projections match canonical skill source"
given = "Canonical skill files and all agent projections exist"
when = "The files are compared byte-for-byte"
then = "Every projection matches its canonical source"
confirmation = "bash -c 'cmp -s skills/gap-closure-contract/SKILL.md .agents/skills/gap-closure-contract/SKILL.md && cmp -s skills/gap-closure-contract/SKILL.md .agents/skills/gap-closure-contract.md && cmp -s skills/gap-closure-contract/SKILL.md .cursor/skills/gap-closure-contract/SKILL.md && cmp -s skills/gap-closure-contract/SKILL.md .claude/skills/gap-closure-contract/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md && cmp -s skills/task-registry-flow/SKILL.md .cursor/skills/task-registry-flow/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md'"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'cmp -s skills/gap-closure-contract/SKILL.md .agents/skills/gap-closure-contract/SKILL.md && cmp -s skills/gap-closure-contract/SKILL.md .agents/skills/gap-closure-contract.md && cmp -s skills/gap-closure-contract/SKILL.md .cursor/skills/gap-closure-contract/SKILL.md && cmp -s skills/gap-closure-contract/SKILL.md .claude/skills/gap-closure-contract/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .agents/skills/task-registry-flow.md && cmp -s skills/task-registry-flow/SKILL.md .cursor/skills/task-registry-flow/SKILL.md && cmp -s skills/task-registry-flow/SKILL.md .claude/skills/task-registry-flow/SKILL.md'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-DOC01"
status = "planned"
kind = "governance"
reason = "Production-facing docs must match the current hardened runtime before release."
behavior_ids = ["B-DOC01-positive", "B-DOC01-negative"]
title = "Update README and system docs"
acceptance_proof = "Behaviors B-DOC01-positive and B-DOC01-negative pass."

[[tasks.targets]]
file = "README.md"
object = "production_contract"
required_change = "Document clean-tree release readiness, manifest-backed source tracking, native files, receipt chains, terminal task immutability, and Nix verification."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "runtime_invariants"
required_change = "Add runtime invariants for receipts, mutation target parsing, terminal task immutability, release source, and final release waivers."

[[tasks.targets]]
file = "docs/releases/v2.md"
object = "release_checklist"
required_change = "Use REQUIREMENTS.toml as artifact authority and list current final release gates."

[[tasks.targets]]
file = "docs/agent-environment-matrix.md"
object = "environment_matrix"
required_change = "Add Claude Code and shared hardening rules across agent environments."

[[tasks]]
task_id = "TASK-2026-05-31-DOC02"
status = "planned"
kind = "governance"
reason = "All agent-facing docs, templates, and skill projections must reinforce the same canonical rules."
behavior_ids = ["B-DOC02-positive", "B-DOC02-negative", "B-DOC02-projection"]
title = "Update agent-facing docs and skill projections"
acceptance_proof = "Behaviors B-DOC02-positive, B-DOC02-negative, and B-DOC02-projection pass."

[[tasks.targets]]
file = "AGENTS.md"
object = "agent_hardening_rules"
required_change = "Add shared production hardening rules for agents."

[[tasks.targets]]
file = "CLAUDE.md"
object = "agent_hardening_rules"
required_change = "Keep Claude Code instructions aligned with shared hardening rules."

[[tasks.targets]]
file = "GEMINI.md"
object = "agent_hardening_rules"
required_change = "Keep Antigravity instructions aligned with shared hardening rules."

[[tasks.targets]]
file = ".cursor/rules/agent-governance.mdc"
object = "cursor_hardening_rules"
required_change = "State Cursor-specific production hardening expectations."

[[tasks.targets]]
file = "templates/AGENTS.md.template"
object = "agent_hardening_rules_template"
required_change = "Render shared hardening rules for future Codex installs."

[[tasks.targets]]
file = "templates/CLAUDE.md.template"
object = "agent_hardening_rules_template"
required_change = "Render shared hardening rules for future Claude Code installs."

[[tasks.targets]]
file = "templates/GEMINI.md.template"
object = "agent_hardening_rules_template"
required_change = "Render shared hardening rules for future Antigravity installs."

[[tasks.targets]]
file = "templates/.cursor/rules/agent-governance.mdc.template"
object = "cursor_hardening_rules_template"
required_change = "Render Cursor hardening rules for future installs."

[[tasks.targets]]
file = "skills/gap-closure-contract/SKILL.md"
object = "contract_requirements"
required_change = "Include documentation sync, terminal task, receipt-chain, and release-source requirements."

[[tasks.targets]]
file = "skills/task-registry-flow/SKILL.md"
object = "registry_flow_requirements"
required_change = "Include terminal immutability and receipt-chain handoff requirements."

[[tasks.targets]]
file = ".agents/skills/gap-closure-contract/SKILL.md"
object = "codex_projection"
required_change = "Mirror canonical gap-closure-contract skill source."

[[tasks.targets]]
file = ".agents/skills/gap-closure-contract.md"
object = "antigravity_markdown_projection"
required_change = "Mirror canonical gap-closure-contract skill source."

[[tasks.targets]]
file = ".cursor/skills/gap-closure-contract/SKILL.md"
object = "cursor_projection"
required_change = "Mirror canonical gap-closure-contract skill source."

[[tasks.targets]]
file = ".claude/skills/gap-closure-contract/SKILL.md"
object = "claude_projection"
required_change = "Mirror canonical gap-closure-contract skill source."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow/SKILL.md"
object = "codex_projection"
required_change = "Mirror canonical task-registry-flow skill source."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow.md"
object = "antigravity_markdown_projection"
required_change = "Mirror canonical task-registry-flow skill source."

[[tasks.targets]]
file = ".cursor/skills/task-registry-flow/SKILL.md"
object = "cursor_projection"
required_change = "Mirror canonical task-registry-flow skill source."

[[tasks.targets]]
file = ".claude/skills/task-registry-flow/SKILL.md"
object = "claude_projection"
required_change = "Mirror canonical task-registry-flow skill source."
```
