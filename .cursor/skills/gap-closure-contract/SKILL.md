---
name: gap-closure-contract
description: Mandatory repo-local workflow for gap closure before approved implementation. Use after analysis, review, verifier failure, audit, or drift report surfaces implementation gaps and the user approves a plan or says to proceed/implement/continue/close the gaps. Before mutating code, create or refresh a closure contract in docs/plans with exact scope, file checklist, per-gap success criteria, validation plan, walkthrough evidence, Task Manifest, PLAN_ACTIVATE, and TASK_REPORT.
---

# Gap Closure Contract

Mandatory after gap analysis plus user approval, before implementation. Scope is the configured project root only.

**Project extensions:** when `PROJECT.md` exists beside this skill, load it after this file for repo-specific paths, CLI commands, architecture, and validation gates.

## Mandatory Approval Hook

- Trigger after gaps are surfaced and the user approves a plan, says proceed, implement, continue, or authorizes gap closure.
- Codex: `.codex/hooks.json` + `.codex/agent-governance.toml`
- Antigravity: `.agents/hooks.json` → mutation hook script → `.codex/scripts/task-registry verify-mutation-hook` (PreToolUse deny on unbound implementation writes).
- Do not mutate implementation files until the closure contract exists or is refreshed for approved scope.
- Do not mutate until the contract has a machine-readable `## Task Manifest` and `PLAN_ACTIVATE` has run via `$task-registry-flow`.
- Partial approval → contract only for approved subset; list deferred gaps explicitly.
- Ambiguous approval → one short clarification before code edits.

## Mandatory Path Isolation

- Target project root: value of `[workspace_boundary].repo_root` in `.codex/agent-governance.toml` (must match `git rev-parse --show-toplevel`).
- Scratch root: `[workspace_boundary].allowed_scratch_root` or `[gap_closure_contract].scratch_root`.
- Before writes or mutating commands: verify `pwd` and `git rev-parse --show-toplevel` match the target root. Stop otherwise.
- Persist contracts at `docs/plans/<short-slug>.md`.
- Scratch only at `<scratch-root>/<short-slug>/task.md`, `walkthrough.md`, `notes.md`.
- Never write contracts or run mutating commands in sibling repos.

## Task Registry Flow

- Approved contracts are tasklist contracts with `## Task Manifest` (one fenced TOML block).
- After approval: `$task-registry-flow` → `.codex/scripts/task-registry activate` (see `.codex/agent-governance.toml` → `[task_registry].cli_command`).
- Status updates use the registry CLI `status` opcode — not `deferred` via status.
- Defer (`TASK_DEFER`): registry CLI `defer` opcode — requires `deferral_governance_basis` and `reactivation_condition` in the registry row.
- Final report: registry CLI `report` opcode.
- Valid statuses: `planned`, `active`, `blocked`, `deferred`, `completed`, `cancelled`
- Permanent non-goals stay in project constitution or vision docs, not the registry.

## Ground Rules

- Follow project authority order in `.codex/agent-governance.toml` and [AGENTS.md](AGENTS.md).
- Respect architecture and product rules defined in project docs — not in this skill.
- Keep source/config/docs/script files at or below 1600 lines. This is a hard plugin rule, not an optional project preference.
- Protect unrelated dirty worktree changes.
- When a closure changes runtime behavior, release posture, or governance rules,
  update README, system docs, agent-facing docs, install templates, and skill
  projections in the same scope.
- Completed and cancelled registry rows are terminal task states. Do not rewrite
  their provenance; changed follow-up work requires a new `task_id`.
- Release-source required files must be native files, not symlinks, and new
  governed files must be declared in `REQUIREMENTS.toml`.
- Mutating task-registry commands must leave an intact receipt chain; include
  `verify-chain` evidence in handoff when registry state changed.

## Contract Requirements

Every contract must include:

1. **Approved Scope** — gaps in/out of scope; deferred items with governed basis + reactivation; schema/API/domain impact when relevant.
2. **Phased Required Change Checklist** — split work into activation/safety, implementation, negative/migration tests, docs/release/handoff as applicable. Every row must use `[NEW]`, `[MODIFY]`, `[DELETE]`, `[GENERATE]`, or `[VERIFY]`, name an exact relative file path, name the object/section changed, and cite acceptance proof.
3. **Per-Gap Success Criteria** — each `###` gap must include current failure, good behavior, forbidden behavior, files involved, positive test, negative test, data/schema/provenance criteria, and runtime criteria or honest `N/A` with reason.
4. **Behavioral Contract (Task Manifest)** — every new active plan manifest must use `schema_version = 2`, declare typed `[[behaviors]]` rows with `gap_id` and `polarity = "positive"|"negative"|"validation"`, and include at least one typed `[[behaviors.verifiers]]` entry per behavior. Each implementation gap must have both positive and negative behavior coverage. `confirmation` remains human-readable context; verifiers are the executable contract. `acceptance_proof` must cite the behavior and verifier command or assertion, not documentation alone.
5. **Validation Plan** — exact runnable commands; focused vs full gates.
6. **Documentation and release sync** — when behavior, release posture, or
   agent workflow changes, list every README, system doc, agent doc, template,
   and skill projection that must change; include negative checks for stale
   claims.
7. **Source File Limit** — state expected line-budget impact and include `.codex/scripts/task-registry source-limit check`; if any file is already over 1600 lines, run `.codex/scripts/task-registry source-limit plan --path <file>` and split before adding more behavior.
8. **Walkthrough Evidence** — proof to capture after implementation (command output, not doc edits), including receipt chain verification when registry receipts changed.
9. **Task Manifest** — fenced `toml` with `schema_version = 2`, `plan_id`, `[[behaviors]]` with `gap_id` and `polarity`, typed `[[behaviors.verifiers]]`, `[[tasks]]` with `behavior_ids`, and diffable `[[tasks.targets]]`. Implementation, schema, authorization, migration, release, and governance tasks must link positive or negative gap behavior, not only validation behavior.

## Closure Workflow

1. Ground gaps in code, tests, specs, contracts, or runtime behavior.
2. Write `docs/plans/<short-slug>.md`; scratch under scratch root if needed.
3. `PLAN_ACTIVATE` before code edits.
4. Patch narrowly; match existing project patterns.
5. Verify: contract tests + project validation gates as applicable.
6. Report proof; include `TASK_REPORT` and `TASK_METRICS`; do not close gaps without meeting criteria unless user waives.

## Contract Template

```markdown
# <Title> Gap Closure Contract

## Approved Scope
...

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[MODIFY]` `path/to/file` - `specific_object`: ...

## Per-Gap Success Criteria
### <Gap>
- Current failure: ...
- Good behavior: Given … when … then … (observable; cite tests or API responses)
- Forbidden behavior: Given … when … then … fails closed
- Files involved: `exact/path.rs`
- Positive test: `<runnable positive command>`
- Negative test: `<runnable negative command>`
- Domain/API/UI: ... or N/A with reason
- Runtime: ... or N/A with reason

## Validation Plan
Focused:
- `<runnable verifier command or assertion gate>`
- `.codex/scripts/task-registry source-limit check`
Full:
- `<project full gate commands>`

## Walkthrough Evidence Required
- ...

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-YYYY-MM-DD-short-slug"

[[behaviors]]
behavior_id = "B-001-short-name"
gap_id = "GAP-001"
polarity = "positive"
title = "Short behavior title"
given = "initial state or fixture"
when = "action under test"
then = "observable outcome asserted in tests"
confirmation = "<runnable test or check command>"

[[behaviors.verifiers]]
type = "command"
command = "<runnable test or check command>"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-short-name-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Short negative behavior title"
given = "invalid state or forbidden input"
when = "the forbidden path is attempted"
then = "the system fails closed with observable evidence"
confirmation = "<runnable negative test or check command>"

[[behaviors.verifiers]]
type = "command"
command = "<runnable negative test or check command>"
expected_exit = 0

[[tasks]]
task_id = "TASK-YYYY-MM-DD-001"
behavior_ids = ["B-001-short-name"]
acceptance_proof = "Behavior B-001-short-name: <same confirmation command>"
...
```
```

## Validation Menu

Use the smallest honest subset from the contract Validation Plan. Prefer commands listed in `.codex/agent-governance.toml` → `[validation]` as starting points when the contract does not specify otherwise.

## Failure Patterns To Catch

- Missing file names or broad targets (“update UI”, “fix backend”).
- Domain logic placed in the wrong layer per project architecture docs.
- Generated artifacts hand-edited when the project treats them as build outputs.
- Verifier passes on static artifacts while runtime/API behavior diverges.
- Wrong repo, wrong scratch root, or commands from a different project's toolchain.
- Missing negative behavior coverage for any implementation gap.
- Behavioral criteria that cite documentation instead of typed verifiers.
- Treating the 1600-line source limit as a final cleanup item instead of a design-time split constraint.
- Updating runtime behavior without synchronizing README, system docs,
  agent-facing docs, templates, and skill projections.
- Reusing completed or cancelled task ids for changed follow-up work.
- Accepting symlinked required release-source files or final-release waiver
  flags as production-ready.
