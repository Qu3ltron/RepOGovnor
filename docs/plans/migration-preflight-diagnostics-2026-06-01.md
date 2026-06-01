# Migration Preflight Diagnostics Gap Closure Contract

## Approved Scope

Close GP-003 by making the local status surface identify common stale v0.x/v1
migration layouts with direct remediation diagnostics. In scope: Rust
`status-check` diagnostics for stale legacy files, both canonical `.agents`
skill projections, install-mode fixtures that exercise those layouts, and the
gap pipeline evidence update.

Out of scope: compatibility shims, restoring removed paths, accepting legacy
schemas, hosted migration support, or changing installer write semantics.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/migration-preflight-diagnostics-2026-06-01.md` - `closure_contract`: declare GP-003 scope, behaviors, validation, and task targets.
- [ ] `[VERIFY]` `.codex/scripts/task-registry activate docs/plans/migration-preflight-diagnostics-2026-06-01.md` - `PLAN_ACTIVATE`: activate exact task targets before implementation edits.

### Phase 1: Status diagnostics
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/status_checks.rs` - `migration_preflight_checks`: report stale legacy paths and both required native `.agents` skill directories with remediation.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/status_check_tests.rs` - `migration_preflight_tests`: assert stale path and native skill failures in JSON diagnostics and assert clean current layout passes.

### Phase 2: Installer fixture coverage
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `old_layout_fixture`: seed additional stale hook/settings layouts and assert merge/force remove them.

### Phase 3: Gap pipeline and handoff
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-003`: replace the open fixture gap with current evidence and remaining reactivation conditions.
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-landing --plan-id PLAN-2026-06-01-migration-preflight-diagnostics --changed-files rust/task-registry-flow-cli/src/status_checks.rs rust/task-registry-flow-cli/src/tests/status_check_tests.rs scripts/test-install-modes.sh docs/gap-pipeline.md` - `TASK_VERIFY_LANDING`: land through behavior verification.

## Per-Gap Success Criteria

### GP-003 Migration Preflight Diagnostics
- Current failure: `status-check` verifies markers and one `.agents` skill but does not provide a compact Rust diagnostic surface for common stale v0.x/v1 files or the full pair of required native skill projections.
- Good behavior: Given a workspace with stale settings, legacy hook paths, stale Antigravity hook, or symlinked `.agents` skills, when `status-check --format json` runs, then each stale or non-native item appears as a typed failure with remediation.
- Forbidden behavior: Given a clean v2 workspace with current markers, no stale paths, and both native `.agents` skill directories, `status-check --format json` must not fail.
- Files involved: `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/tests/status_check_tests.rs`, `scripts/test-install-modes.sh`, `docs/gap-pipeline.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_reports_stale_migration_layouts_with_remediation`
- Data/schema/provenance: Diagnostics remain schema-backed `CheckReport` JSON with stable `check_id`, `path`, `expected`, `actual`, and `remediation` fields.
- Runtime: `scripts/test-install-modes.sh` proves merge and force remove the seeded stale layouts without preserving compatibility paths.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_`
- `bash scripts/test-install-modes.sh`
- `.codex/scripts/task-registry source-limit check`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `.codex/scripts/task-registry validate`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected line impact is small and remains below the 1600-line cap. Validate with
`.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Focused Rust status-check tests pass.
- Install-mode fixture passes with seeded stale legacy layouts removed by merge and force.
- `TASK_VERIFY_LANDING` completes the task.
- Registry report, metrics, validation, source-limit check, and receipt-chain verification pass.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-migration-preflight-diagnostics"

[[behaviors]]
behavior_id = "B-2026-06-01-migration-preflight-positive"
gap_id = "GP-003"
polarity = "positive"
title = "Clean v2 status preflight passes"
given = "A workspace with current marker docs, no stale legacy paths, and both native .agents skills"
when = "status-check --format json runs"
then = "The status report exits zero with no failures"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-migration-preflight-negative"
gap_id = "GP-003"
polarity = "negative"
title = "Stale migration layouts fail with remediation"
given = "A workspace containing legacy settings, hook files, stale Antigravity hook paths, and symlinked .agents skill projections"
when = "status-check --format json runs"
then = "The status report fails with typed stale-path and native-skill diagnostics that include remediation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_reports_stale_migration_layouts_with_remediation"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_reports_stale_migration_layouts_with_remediation"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-06-01-migration-fixture-validation"
gap_id = "GP-003"
polarity = "validation"
title = "Install fixture removes seeded stale layouts"
given = "The install-mode smoke fixture with common stale v0.x and v1 paths"
when = "merge and force installs run"
then = "Stale paths are removed and the fixture exits successfully"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-migration-preflight-001"
status = "planned"
title = "Add migration preflight diagnostics"
kind = "migration"
reason = "GP-003 needs targeted old-layout diagnostics before broader public migration."
acceptance_proof = "Behaviors B-2026-06-01-migration-preflight-positive and B-2026-06-01-migration-preflight-negative pass."
behavior_ids = [
  "B-2026-06-01-migration-preflight-positive",
  "B-2026-06-01-migration-preflight-negative",
  "B-2026-06-01-migration-fixture-validation",
]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "migration_preflight_checks"
required_change = "Report stale legacy paths and both required native .agents skill projections."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/status_check_tests.rs"
object = "migration_preflight_tests"
required_change = "Assert clean v2 success and stale old-layout remediation failures."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "old_layout_fixture"
required_change = "Seed additional stale hook/settings layouts and assert merge/force removal."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-003"
required_change = "Update GP-003 with current migration preflight evidence and remaining reactivation conditions."
```
