# Project extensions for task-registry-flow

Optional. Copy to `PROJECT.md` beside the installed skill and edit for your repository.

## Path and executor

- Repository: `<absolute-path>`
- Executor: `.codex/scripts/task-registry`

## Opcode commands

| Opcode | Command |
|--------|---------|
| `PLAN_ACTIVATE` | `<cli> activate <docs/plans/file.md>` |
| `TASK_STATUS` | `<cli> status <task_id> <status>` |
| `TASK_DEFER` | `<cli> defer <task_id> <basis> <reactivation_condition>` |
| `TASK_REPORT` | `<cli> report <plan_id>` |
| `TASK_VALIDATE` | `<cli> validate` |
| `TASK_VERIFY_BEHAVIORS` | `<cli> verify-behaviors <plan_id-or-task_id>` |
| `TASK_METRICS` | `<cli> metrics` |

## Project-specific rules

- CI integration, feature registry, or deferral bases unique to this repo.
