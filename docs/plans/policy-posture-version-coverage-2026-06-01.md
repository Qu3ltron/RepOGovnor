# Policy Posture Version Coverage Gap Closure Contract

## Approved Scope

Close the release-governance coverage gap opened by the completed policy posture
work. `version-check validate` requires every completed post-cutover plan to be
covered by `docs/version-roadmap.toml`.

In scope:
- Add the completed policy posture plan to `2.1.0` covered plan ids.
- Add this follow-up plan to the same covered plan ids before completing it.

Out of scope:
- Changing release version, tag, prerelease tag, or push policy.
- Changing product posture docs beyond release coverage metadata.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/policy-posture-version-coverage-2026-06-01.md` - `closure_contract`: create and activate this contract before editing release coverage.
- [ ] `[VERIFY]` `docs/plans/policy-posture-version-coverage-2026-06-01.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/policy-posture-version-coverage-2026-06-01.md`.

### Phase 1: Version coverage
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `covered_plan_ids`: add `PLAN-2026-06-01-policy-compliance-cost-posture` and `PLAN-2026-06-01-policy-posture-version-coverage`.

### Phase 2: Verification and landing
- [ ] `[VERIFY]` `docs/version-roadmap.toml` - `version_check`: `.codex/scripts/task-registry version-check validate`.
- [ ] `[VERIFY]` `docs/version-roadmap.toml` - `missing_coverage_absent`: `bash -lc '.codex/scripts/task-registry version-check validate 2>&1 | tee /tmp/repogovnor-version-coverage.out >/dev/null; ! rg -n "missing" /tmp/repogovnor-version-coverage.out'`.

## Per-Gap Success Criteria

### GAP-001: Completed policy posture plan is missing release coverage
- Current failure: `version-check validate` fails with missing release coverage for `PLAN-2026-06-01-policy-compliance-cost-posture`.
- Good behavior: Given the updated roadmap, when `version-check validate` runs, then release coverage passes.
- Forbidden behavior: The update must not change release version, tag, prerelease tag, or push policy.
- Files involved: `docs/version-roadmap.toml`.
- Positive test: `.codex/scripts/task-registry version-check validate`.
- Negative test: `bash -lc '.codex/scripts/task-registry version-check validate 2>&1 | tee /tmp/repogovnor-version-coverage.out >/dev/null; ! rg -n "missing" /tmp/repogovnor-version-coverage.out'`.
- Domain/API/UI: Release metadata only.
- Runtime: Version governance must pass after landing.

## Validation Plan

Focused:
- `.codex/scripts/task-registry version-check validate`
- `bash -lc '.codex/scripts/task-registry version-check validate 2>&1 | tee /tmp/repogovnor-version-coverage.out >/dev/null; ! rg -n "missing" /tmp/repogovnor-version-coverage.out'`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `.codex/scripts/task-registry backlog-check`
- `.codex/scripts/task-registry status-check --format json`
- `scripts/status.sh --release-source`

## Source File Limit

This is a small TOML metadata change. Final verification includes
`.codex/scripts/task-registry source-limit check`.

## Walkthrough Evidence

- Plan activation output.
- Version-check output.
- Missing-coverage negative check output.
- Source-limit and registry validation output.
- `TASK_REPORT` and `TASK_METRICS` after landing.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-policy-posture-version-coverage"

[[behaviors]]
behavior_id = "B-001-version-coverage-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Version check validates release coverage"
given = "The 2.1.0 roadmap covered plan ids include the completed policy posture work and this follow-up"
when = "version-check validate runs"
then = "The version governance report passes"
confirmation = ".codex/scripts/task-registry version-check validate"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry version-check validate"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-version-coverage-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Version check has no missing coverage diagnostics"
given = "The 2.1.0 roadmap covered plan ids include all completed post-cutover plans"
when = "version-check validate output is inspected"
then = "No missing coverage diagnostic remains"
confirmation = "bash -lc '.codex/scripts/task-registry version-check validate 2>&1 | tee /tmp/repogovnor-version-coverage.out >/dev/null; ! rg -n \"missing\" /tmp/repogovnor-version-coverage.out'"

[[behaviors.verifiers]]
type = "command"
command = "bash -lc '.codex/scripts/task-registry version-check validate 2>&1 | tee /tmp/repogovnor-version-coverage.out >/dev/null; ! rg -n \"missing\" /tmp/repogovnor-version-coverage.out'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-policy-posture-version-coverage-001"
title = "Cover policy posture work in version roadmap"
kind = "release"
status = "planned"
reason = "Completed post-cutover plans must be represented in release coverage before 2.1.0 validation can pass."
behavior_ids = ["B-001-version-coverage-positive", "B-002-version-coverage-negative"]
acceptance_proof = "Behaviors B-001 and B-002 pass their typed verifiers."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "covered_plan_ids"
required_change = "Add policy posture and version coverage plan ids to the 2.1.0 release coverage list."
```
