# First-Run Installer Guidance Gap Closure Contract

## Approved Scope

Close GP-001 by making installer output give first-time users a direct next
action after dry-run and after an applied install. In scope:
`scripts/install-to-workspace.sh` output, install-mode fixture assertions, and
the gap pipeline evidence update.

Out of scope: hosted onboarding, compatibility shims, changing install modes,
sample repository creation, or weakening validation gates.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/first-run-installer-guidance-2026-06-01.md` - `closure_contract`: declare GP-001 scope, success criteria, validation, and task targets.
- [ ] `[VERIFY]` `.codex/scripts/task-registry activate docs/plans/first-run-installer-guidance-2026-06-01.md` - `PLAN_ACTIVATE`: activate exact task targets before implementation edits.

### Phase 1: Installer output
- [ ] `[MODIFY]` `scripts/install-to-workspace.sh` - `first_run_next_steps`: add explicit dry-run continuation, applied-install validation commands, and docs pointers for first workflow and migration.

### Phase 2: Output fixtures
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `first_run_output_assertions`: assert dry-run and applied install output contain the first-run path and canonical validation commands.

### Phase 3: Gap pipeline
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-001`: update current evidence and remaining reactivation conditions.
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-landing --plan-id PLAN-2026-06-01-first-run-installer-guidance --changed-files scripts/install-to-workspace.sh scripts/test-install-modes.sh docs/gap-pipeline.md` - `TASK_VERIFY_LANDING`: land through behavior verification.

## Per-Gap Success Criteria

### GP-001 First-Run Installer Guidance
- Current failure: first-run install output lists actions and validation, but it does not clearly tell a new user what to do after dry-run or point an applied install toward the workflow and migration docs.
- Good behavior: Given a dry-run install, output says no files changed and tells the user to rerun with `--merge` or `--force`. Given an applied install, output includes first-run next steps, canonical validation commands, and direct docs pointers.
- Forbidden behavior: A dry-run must not imply files were applied or that posture has already passed.
- Files involved: `scripts/install-to-workspace.sh`, `scripts/test-install-modes.sh`, `docs/gap-pipeline.md`.
- Positive test: `bash scripts/test-install-modes.sh`
- Negative test: `bash -c 'bash scripts/test-install-modes.sh && grep -q "Dry run only; no files changed." <(plugins/agent-governance/scripts/install-to-workspace.sh --help 2>/dev/null) || true'` is not used as proof; the actual negative is the fixture hash check plus dry-run output assertions in `scripts/test-install-modes.sh`.
- Data/schema/provenance: Output remains local shell text only; JSON dry-run output from `render-from-config.sh` is unchanged.
- Runtime: Install-mode smoke proves dry-run is non-mutating and applied installs emit validation guidance.

## Validation Plan

Focused:
- `bash scripts/test-install-modes.sh`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected line impact is small and remains below the 1600-line cap. Validate with
`.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Dry-run fixture remains non-mutating and asserts rerun guidance.
- Merge and force fixtures assert first-run next steps and validation commands.
- `TASK_VERIFY_LANDING` completes the task.
- Registry report, metrics, validation, source-limit check, and receipt-chain verification pass.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-first-run-installer-guidance"

[[behaviors]]
behavior_id = "B-2026-06-01-first-run-positive"
gap_id = "GP-001"
polarity = "positive"
title = "Applied install prints first-run next steps"
given = "An install-mode fixture running merge and force installs"
when = "install-to-workspace.sh applies the projection"
then = "Output includes first-run next steps, canonical validation commands, and workflow or migration doc pointers"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-first-run-negative"
gap_id = "GP-001"
polarity = "negative"
title = "Dry run stays preview-only"
given = "An install-mode fixture running --dry-run"
when = "install-to-workspace.sh projects changes"
then = "Output says no files changed and tells the user to rerun with --merge or --force"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-first-run-installer-guidance-001"
status = "planned"
title = "Clarify first-run installer output"
kind = "documentation"
reason = "GP-001 needs direct first-run guidance in the command output new public users see."
acceptance_proof = "Behaviors B-2026-06-01-first-run-positive and B-2026-06-01-first-run-negative pass."
behavior_ids = [
  "B-2026-06-01-first-run-positive",
  "B-2026-06-01-first-run-negative",
]

[[tasks.targets]]
file = "scripts/install-to-workspace.sh"
object = "first_run_next_steps"
required_change = "Print explicit dry-run continuation and applied-install first-run validation guidance."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "first_run_output_assertions"
required_change = "Assert installer output includes dry-run continuation and applied-install next steps."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-001"
required_change = "Update GP-001 with current first-run installer evidence and remaining reactivation conditions."
```
