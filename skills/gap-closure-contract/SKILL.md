---
name: gap-closure-contract
description: Mandatory repo-local workflow for gap closure before approved implementation. Use after analysis, review, verifier failure, audit, or drift report surfaces implementation gaps and the user approves a plan or says to proceed/implement/continue/close the gaps. Before mutating code, create or refresh a closure contract in docs/plans with exact scope, file checklist, per-gap success criteria, validation plan, walkthrough evidence, Task Manifest, PLAN_ACTIVATE, and TASK_REPORT.
---

# Gap Closure Contract

Mandatory after gap analysis plus user approval, before implementation. Scope is the configured project root only.

**Project extensions:** when `PROJECT.md` exists beside this skill, load it after this file for repo-specific paths, CLI commands, architecture, and validation gates.

## Mandatory Approval Hook

- Trigger after gaps are surfaced and the user approves a plan, says proceed, implement, continue, or authorizes gap closure.
- Codex: `.codex/hooks/user-plan-approval.toml`
- Antigravity: `.agents/hooks.json` → mutation hook script → project `verify-mutation-hook` command (PreToolUse deny on unbound implementation writes when the executor enforces it).
- Do not mutate implementation files until the closure contract exists or is refreshed for approved scope.
- Do not mutate until the contract has a machine-readable `## Task Manifest` and `PLAN_ACTIVATE` has run via `$task-registry-flow`.
- Partial approval → contract only for approved subset; list deferred gaps explicitly.
- Ambiguous approval → one short clarification before code edits.

## Mandatory Path Isolation

- Target project root: value of `[workspace_boundary].repo_root` in `.codex/settings.toml` (must match `git rev-parse --show-toplevel`).
- Scratch root: `[workspace_boundary].allowed_scratch_root` or `[gap_closure_contract].scratch_root`.
- Before writes or mutating commands: verify `pwd` and `git rev-parse --show-toplevel` match the target root. Stop otherwise.
- Persist contracts at `docs/plans/<short-slug>.md`.
- Scratch only at `<scratch-root>/<short-slug>/task.md`, `walkthrough.md`, `notes.md`.
- Never write contracts or run mutating commands in sibling repos.

## Task Registry Flow

- Approved contracts are tasklist contracts with `## Task Manifest` (one fenced TOML block).
- After approval: `$task-registry-flow` → registry CLI `activate` opcode (see `.codex/settings.toml` → `[task_registry].cli_command`).
- Status updates use the registry CLI `status` opcode — not `deferred` via status.
- Defer (`TASK_DEFER`): registry CLI `defer` opcode — requires `deferral_governance_basis` and `reactivation_condition` in the registry row.
- Final report: registry CLI `report` opcode.
- Valid statuses: `planned`, `active`, `blocked`, `deferred`, `completed`, `cancelled`
- Permanent non-goals stay in project constitution or vision docs, not the registry.

## Ground Rules

- Follow project authority order in `.codex/settings.toml` and [AGENTS.md](AGENTS.md).
- Respect architecture and product rules defined in project docs — not in this skill.
- Keep source/config/docs/script files within project line limits when configured.
- Protect unrelated dirty worktree changes.

## Contract Requirements

Every contract must include:

1. **Approved Scope** — gaps in/out of scope; deferred items with governed basis + reactivation; schema/API/domain impact when relevant.
2. **Required Change Checklist** — `[NEW]`, `[MODIFY]`, `[DELETE]`, `[GENERATE]` with exact paths and acceptance proof.
3. **Per-Gap Success Criteria** — each `###` gap must include a `- Behavioral:` line describing observable **given/when/then** outcomes (not documentation). Add domain-specific criteria (API, UI, runtime, visual) when touched; honest `N/A` with reason when not.
4. **Behavioral Contract (Task Manifest)** — every active plan manifest must declare typed `[[behaviors]]` rows (`behavior_id`, `title`, `given`, `when`, `then`, `confirmation`) and link each `[[tasks]]` row through `behavior_ids`. `confirmation` must be a **runnable** command (tests, linters, integration scripts). `acceptance_proof` must cite the behavior and command, not documentation alone.
5. **Validation Plan** — exact runnable commands; focused vs full gates.
6. **Walkthrough Evidence** — proof to capture after implementation (command output, not doc edits).
7. **Task Manifest** — fenced `toml` with `schema_version`, `plan_id`, `[[behaviors]]`, `[[tasks]]` (with `behavior_ids`), and diffable `[[tasks.targets]]`.

## Closure Workflow

1. Ground gaps in code, tests, specs, contracts, or runtime behavior.
2. Write `docs/plans/<short-slug>.md`; scratch under scratch root if needed.
3. `PLAN_ACTIVATE` before code edits.
4. Patch narrowly; match existing project patterns.
5. Verify: contract tests + project validation gates as applicable.
6. Report proof; include `TASK_REPORT`; do not close gaps without meeting criteria unless user waives.

## Contract Template

```markdown
# <Title> Gap Closure Contract

## Approved Scope
...

## Required Change Checklist
- [ ] `[MODIFY]` `path/to/file` - ...

## Per-Gap Success Criteria
### <Gap>
- Behavioral: Given … when … then … (observable; cite tests or API responses)
- Domain/API/UI: ... or N/A with reason
- Runtime: ... or N/A with reason

## Validation Plan
Focused:
- `<runnable confirmation command>`
Full:
- `<project full gate commands>`

## Walkthrough Evidence Required
- ...

## Task Manifest
```toml
schema_version = 1
plan_id = "PLAN-YYYY-MM-DD-short-slug"

[[behaviors]]
behavior_id = "B-001-short-name"
title = "Short behavior title"
given = "initial state or fixture"
when = "action under test"
then = "observable outcome asserted in tests"
confirmation = "<runnable test or check command>"

[[tasks]]
task_id = "TASK-YYYY-MM-DD-001"
behavior_ids = ["B-001-short-name"]
acceptance_proof = "Behavior B-001-short-name: <same confirmation command>"
...
```
```

## Validation Menu

Use the smallest honest subset from the contract Validation Plan. Prefer commands listed in `.codex/settings.toml` → `[validation]` as starting points when the contract does not specify otherwise.

## Failure Patterns To Catch

- Missing file names or broad targets (“update UI”, “fix backend”).
- Domain logic placed in the wrong layer per project architecture docs.
- Generated artifacts hand-edited when the project treats them as build outputs.
- Verifier passes on static artifacts while runtime/API behavior diverges.
- Wrong repo, wrong scratch root, or commands from a different project's toolchain.
- Behavioral criteria that cite documentation instead of runnable confirmations.
