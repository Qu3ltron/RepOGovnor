---
name: task-registry-flow
description: Deterministic repo-local workflow for activating approved implementation contracts into docs/task-registry.toml, updating task statuses, deferring tasks, and reporting blocked or deferred work at implementation handoff. Use after a docs/plans contract is approved or when any task status must change.
---

# Task Registry Flow

Manages `docs/task-registry.toml` — authoritative status ledger for approved plan tasks in the configured project root.

**Project extensions:** when `PROJECT.md` exists beside this skill, load it after this file for repo-specific executor commands and policy.

**Executor:** plugin-owned Rust CLI at `.codex/scripts/task-registry` from `.codex/agent-governance.toml` → `[task_registry].cli_command`. Do not use project-native `task_registry` binaries or hand-edit the registry when the CLI can perform the operation.

## Required Opcodes

| Opcode | Purpose |
|--------|---------|
| `PLAN_ACTIVATE` | Activate a plan from `docs/plans/<file>.md` |
| `TASK_STATUS` | Set non-terminal status (`planned`, `active`, `blocked`, `cancelled`) |
| `TASK_DEFER` | Defer with governed basis and reactivation condition |
| `TASK_REPORT` | Final handoff report for a plan |
| `TASK_ARCHIVE_COMPLETED` | Move completed history to archive when supported |
| `TASK_VALIDATE` | Typed TOML validation of registry + agent paths |
| `TASK_VERIFY_BEHAVIORS` | Run linked typed behavior verifiers |
| `TASK_VERIFY_LANDING` | Bind changed files to active task targets, run verifiers, and write completed status |
| `TASK_VERIFY_MUTATION_HOOK` | Deny unbound implementation writes through the hook |
| `TASK_METRICS` | Summarize local workflow receipts and registry state |
| `SOURCE_LIMIT_CHECK` | Enforce the 1600-line source/governance file limit |
| `SOURCE_LIMIT_PLAN` | Produce deterministic split guidance for violating files |
| `VERIFY_CHAIN` | Validate and repair the receipt hash chain in events.jsonl |

Exact command strings live in `.codex/agent-governance.toml`; the canonical pattern is `.codex/scripts/task-registry activate|status|defer|report|validate|archive-completed|verify-behaviors|verify-landing|verify-mutation-hook|metrics|source-limit ...`.

Do not set `deferred` via `TASK_STATUS`. Use `TASK_DEFER` only with governed basis and exact reactivation condition. Do not set `completed` via `TASK_STATUS`; `TASK_VERIFY_LANDING` owns completed status.

## Activation Rules

- Plan under `docs/plans/` with exactly one `## Task Manifest` fenced TOML block.
- New activations require `schema_version = 2`; legacy `schema_version = 1` manifests may remain only as completed or cancelled archive evidence.
- Manifest parsing and registry validation use typed TOML validation through the registry CLI — no ad hoc string checks or hardcoded plan lists.
- `plan_hash_sha256`: normalized plan text (CRLF/CR → LF, strip trailing whitespace per line, one final newline).
- Registry is authoritative after activation; plans remain intent + hash provenance.
- Never silently delete registry tasks; cancel only when contract or user explicitly cancels.
- Plan hash change after activation → reconcile before continuing implementation.
- Duplicate `plan_id` in registry or across manifests fails validation.
- Every task row must carry matching `plan_id`, `source_plan_path`, `source_plan_hash_sha256`.
- Completed history may live in `docs/task-registry/archive/*.toml`; the plugin CLI supports `archive-completed`.
- Completed and cancelled tasks are terminal task states. Reactivating an
  unchanged plan is idempotent, but changed provenance, behavior ids, targets,
  blockers, or projected steps require a new `task_id`.

## Task Requirements

Each row: `task_id`, `plan_id`, `status`, `title`, `kind`, `reason`, `acceptance_proof`, `source_plan_path`, `source_plan_hash_sha256`, and ≥1 `targets` entry (`file`, `object`, `required_change`).

Active plans must declare `[[behaviors]]` in the plan manifest with `gap_id`, `polarity`, and typed `[[behaviors.verifiers]]` entries. Each implementation gap needs at least one `positive` and one `negative` behavior. Each task lists `behavior_ids`. `verify-landing --plan-id <plan_id> --changed-files <paths>` maps changed files to active task targets, runs those verifiers, and writes completed status.

Implementation, schema, authorization, migration, release, and governance tasks cannot be backed only by `validation` behavior. Full-repo validation is supplementary proof, not the closure itself.

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

Run `.codex/scripts/task-registry verify-landing --plan-id <plan_id> --changed-files <paths>` before final report when implementation tasks should complete. Run `.codex/scripts/task-registry metrics` after substantial implementation work to capture local efficacy evidence from `docs/task-registry/events.jsonl`. Receipts are local only; do not send telemetry.

Run `.codex/scripts/task-registry verify-chain --format json` before production
handoff when registry state changed. The receipt chain must remain intact;
malformed, unchained, or tampered events are release blockers.

Run `.codex/scripts/task-registry source-limit check` before marking implementation tasks complete. If it fails, run `.codex/scripts/task-registry source-limit plan --path <file>` and split the file through an approved contract.

## Related Skills

- Contract authoring: `$gap-closure-contract`
- Project governance: [AGENTS.md](AGENTS.md) and paths in `.codex/agent-governance.toml`
