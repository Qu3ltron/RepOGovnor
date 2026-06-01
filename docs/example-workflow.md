# Minimal Governance Workflow

This example shows the smallest useful Agent Governance loop in a consumer
repository after install.

## 1. Write A Closure Plan

Create `docs/plans/example-change.md` with the required sections:

- Approved Scope
- Phased Required Change Checklist
- Per-Gap Success Criteria
- Validation Plan
- Walkthrough Evidence
- Task Manifest

The Task Manifest must use `schema_version = 2`, exact task targets, typed
behaviors, and positive plus negative behavior coverage for implementation
gaps.

## 2. Activate The Plan

```bash
.codex/scripts/task-registry activate docs/plans/example-change.md
```

Activation writes the plan tasks into `docs/task-registry.toml`. After this
step, implementation writes should stay inside the activated task targets.

## 3. Make The Scoped Edit

Edit only files named in the active task targets. If the needed file is not in
the plan, update the plan and reactivate before editing that file.

## 4. Run Focused Validation

Run the commands listed in the plan's Validation Plan. For most small changes,
include:

```bash
.codex/scripts/task-registry source-limit check
.codex/scripts/task-registry validate
```

Use project-specific tests for the changed behavior.

## 5. Land Through Changed Files

Complete tasks with `verify-landing`, not with direct completed-status writes:

```bash
.codex/scripts/task-registry verify-landing \
  --plan-id PLAN-YYYY-MM-DD-example-change \
  --changed-files path/to/changed-file
```

`verify-landing` binds changed files to active task targets and runs the linked
behavior verifiers before marking tasks completed.

## 6. Report And Archive

```bash
.codex/scripts/task-registry report PLAN-YYYY-MM-DD-example-change
.codex/scripts/task-registry metrics
.codex/scripts/task-registry verify-chain --format json
.codex/scripts/task-registry archive-completed
```

The report is the handoff summary. Metrics and receipt-chain checks show local
workflow state; they do not prove product correctness by themselves.

## 7. Commit The Evidence

Commit the implementation, plan, registry updates, and receipt updates as one
coherent change unless the work naturally splits into separate reviewable
commits.
