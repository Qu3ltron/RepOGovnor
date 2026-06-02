# Token Capture Review Version Coverage Gap Closure Contract

## Approved Scope
Close the release-governance gap surfaced during token capture review validation: the completed review-closure plan must be represented in the canonical version roadmap before release gates can pass. Scope is limited to `docs/version-roadmap.toml` coverage for `PLAN-2026-06-02-token-capture-review-gap-closure` and this follow-up governance plan. No version bump, release tag, API behavior, or pricing behavior changes are in scope.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [x] `[VERIFY]` `docs/plans/token-capture-review-version-coverage-2026-06-02.md` - `Task Manifest`: define typed positive and negative version coverage behaviors before editing release governance.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `PLAN_ACTIVATE`: activate this contract through `.codex/scripts/task-registry activate`.

### Phase 1: Version coverage
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `releases[2.1.0].covered_plan_ids`: add `PLAN-2026-06-02-token-capture-review-gap-closure` and `PLAN-2026-06-02-token-capture-review-version-coverage`.

### Phase 2: Verification and handoff
- [ ] `[VERIFY]` `docs/version-roadmap.toml` - `version-check validate`: prove no completed plan is uncovered.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli/src/tests/version_check_tests.rs` - `version_governance_rejects_uncovered_completed_plan`: prove missing completed-plan coverage still fails closed.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `TASK_REPORT`: report completed governance state.

## Per-Gap Success Criteria
### GAP-001: Completed Token Capture Review Plan Missing From Version Roadmap
- Current failure: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml` fails `version-completed-plan-covered` because `PLAN-2026-06-02-token-capture-review-gap-closure` is completed but absent from `docs/version-roadmap.toml`.
- Good behavior: Given completed task-registry history, when version governance validates, then every completed plan is covered by the current release roadmap.
- Forbidden behavior: A completed plan remains absent from `covered_plan_ids`, or this follow-up plan completes and recursively creates a new uncovered-plan failure.
- Files involved: `docs/version-roadmap.toml`.
- Positive test: `.codex/scripts/task-registry version-check validate --format json`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_uncovered_completed_plan`.
- Domain/API/UI: Release governance only; no user-facing API or pricing schema change.
- Runtime: N/A; this closes deterministic governance metadata, not runtime execution.

## Validation Plan
Focused:
- `.codex/scripts/task-registry version-check validate --format json`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_uncovered_completed_plan`
- `.codex/scripts/task-registry source-limit check`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry release-check all --format json`

## Walkthrough Evidence
- `version-check validate` reports no missing completed plan.
- Focused negative unit test still passes.
- Full cargo and release readiness gates pass after coverage is updated.
- `TASK_REPORT` and `TASK_METRICS` are captured for this plan.

## Source File Limit
The only edited source-of-truth file is `docs/version-roadmap.toml`, currently far below the 1600-line limit. Run `.codex/scripts/task-registry source-limit check` before completion.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-token-capture-review-version-coverage"

[[behaviors]]
behavior_id = "B-TCR-VC-001-version-coverage-valid"
gap_id = "GAP-001"
polarity = "positive"
title = "Completed token capture review plans are release covered"
given = "the token capture review closure and version-coverage closure plans are completed"
when = "version governance validates the current release roadmap"
then = "version-completed-plan-covered passes with no missing completed plan"
confirmation = ".codex/scripts/task-registry version-check validate --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry version-check validate --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-VC-002-version-coverage-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Uncovered completed plans still fail closed"
given = "a fixture registry contains a completed plan not listed in the version roadmap"
when = "version governance validates the fixture"
then = "the version-completed-plan-covered check fails for the missing plan"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_uncovered_completed_plan"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml version_governance_rejects_uncovered_completed_plan"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-review-version-coverage-001"
status = "planned"
title = "Cover token capture review closure in version roadmap"
kind = "governance"
reason = "Completed token capture review closure must be included in release version governance before release readiness can pass."
acceptance_proof = "Behaviors B-TCR-VC-001-version-coverage-valid and B-TCR-VC-002-version-coverage-negative: version-check validate and uncovered-plan negative test pass."
behavior_ids = ["B-TCR-VC-001-version-coverage-valid", "B-TCR-VC-002-version-coverage-negative"]

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "releases[2.1.0].covered_plan_ids"
required_change = "Add PLAN-2026-06-02-token-capture-review-gap-closure and PLAN-2026-06-02-token-capture-review-version-coverage."
```
