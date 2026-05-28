---
name: task-registry-flow
description: Deterministic repo-local workflow for activating approved implementation contracts into docs/task-registry.toml, updating task statuses, deferring tasks, and reporting blocked or deferred work at implementation handoff. Use after a docs/plans contract is approved or when any task status must change.
---

# Task Registry Flow

Manages `docs/task-registry.toml` — authoritative status ledger for approved plan tasks in the configured project root.

**Project extensions:** when `PROJECT.md` exists beside this skill, load it after this file for repo-specific executor commands and policy.

**Executor:** registry CLI from `.codex/settings.toml` → `[task_registry].cli_command` — do not hand-edit the registry when the CLI can perform the operation.

## Required Opcodes

| Opcode | Purpose |
|--------|---------|
| `PLAN_ACTIVATE` | Activate a plan from `docs/plans/<file>.md` |
| `TASK_STATUS` | Set status (`planned`, `active`, `blocked`, `completed`, `cancelled`) |
| `TASK_DEFER` | Defer with governed basis and reactivation condition |
| `TASK_REPORT` | Final handoff report for a plan |
| `TASK_ARCHIVE_COMPLETED` | Move completed history to archive when supported |
| `TASK_VALIDATE` | Typed TOML validation of registry + agent paths |
| `TASK_VERIFY_BEHAVIORS` | Run linked behavior confirmations |

Exact command strings live in `.codex/settings.toml` and the registry CLI skill path. Typical pattern: `<cli_command> activate|status|defer|report|validate|verify-behaviors ...`.

Do not set `deferred` via `TASK_STATUS`. Use `TASK_DEFER` only with governed basis and exact reactivation condition.

## Activation Rules

- Plan under `docs/plans/` with exactly one `## Task Manifest` fenced TOML block.
- Manifest parsing and registry validation use typed TOML validation through the registry CLI — no ad hoc string checks or hardcoded plan lists.
- `plan_hash_sha256`: normalized plan text (CRLF/CR → LF, strip trailing whitespace per line, one final newline).
- Registry is authoritative after activation; plans remain intent + hash provenance.
- Never silently delete registry tasks; cancel only when contract or user explicitly cancels.
- Plan hash change after activation → reconcile before continuing implementation.
- Duplicate `plan_id` in registry or across manifests fails validation.
- Every task row must carry matching `plan_id`, `source_plan_path`, `source_plan_hash_sha256`.
- Completed history may live in `docs/task-registry/archive/*.toml` when the executor supports archives.

## Task Requirements

Each row: `task_id`, `plan_id`, `status`, `title`, `kind`, `reason`, `acceptance_proof`, `source_plan_path`, `source_plan_hash_sha256`, and ≥1 `targets` entry (`file`, `object`, `required_change`).

Active plans must declare `[[behaviors]]` in the plan manifest with runnable `confirmation` commands. Each task lists `behavior_ids`. Marking a task `completed` should run those confirmations and fail if checks fail.

Statuses: `planned`, `active`, `blocked`, `deferred`, `completed`, `cancelled`.

`deferred` only for actionable work intentionally outside current approved scope under a governance rule (constitution, spec authority, missing approval, external blocker, etc.).

## Governed Deferral

`TASK_DEFER` forbidden for inconvenience, size, or vague postponement.

Every `deferred` task needs:

- `deferral_governance_basis`
- `reactivation_condition`

External-data deferrals also need `[[tasks.blockers]]` and `[[tasks.projected_steps]]` with concrete objects, evidence, and unblock conditions when applicable.

## Implementation Handoff

Include:

```text
Task registry: <completed> completed, <deferred> deferred, <blocked> blocked for <plan_id>.
```

List every `deferred` and `blocked` task with `task_id`, title, and reason. Do not claim full closure while tasks remain `planned`, `active`, `blocked`, or `deferred` unless the user explicitly accepts that state.

## Related Skills

- Contract authoring: `$gap-closure-contract`
- Project governance: [AGENTS.md](AGENTS.md) and paths in `.codex/settings.toml`
