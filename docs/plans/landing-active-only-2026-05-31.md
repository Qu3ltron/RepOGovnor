# Active-Only Verify-Landing Gap Closure Contract

## Approved Scope

Close the review gap where `verify-landing` can select a `planned` task target,
run behavior verifiers, and only then fail at the terminal transition.

In scope:

- Restrict landing completion candidates to `active` tasks before verifier
  execution.
- Add a negative test proving a `planned` target is rejected before behavior
  verifiers run.
- Correct runtime documentation and landing error text to describe active-only
  completion.

Out of scope:

- Changing mutation-hook authorization, which may still consider planned and
  active task targets.
- Supporting `planned -> completed`; the current transition model forbids it.

## Phased Required Change Checklist

### Phase 0: Activation

- [ ] `[NEW]` `docs/plans/landing-active-only-2026-05-31.md` - `closure contract`: define active-only landing scope, success criteria, validation, and manifest; acceptance proof is `PLAN_ACTIVATE docs/plans/landing-active-only-2026-05-31.md`.

### Phase 1: Implementation

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/landing.rs` - `landing task selector`: select only active task targets before running verifiers and update failure text; acceptance proof is behaviors `B-2026-05-31-active-landing-G01-positive` and `B-2026-05-31-active-landing-G01-negative`.

### Phase 2: Tests and docs

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/landing_tests.rs` - `planned target rejection`: add a negative test that planned targets fail before verifier execution; acceptance proof is behavior `B-2026-05-31-active-landing-G01-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `failing verifier setup`: activate the sample task before asserting verifier failure; acceptance proof is full cargo test.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Landing Completion Contract`: document active-only task target binding; acceptance proof is behavior `B-2026-05-31-active-landing-G02-negative`.

### Phase 3: Validation and handoff

- [ ] `[VERIFY]` `rust/task-registry-flow-cli/Cargo.toml` - `focused and full gates`: run focused landing tests, source-limit, registry validation, cargo tests, release readiness, receipt chain, report, metrics, and archive; acceptance proof is behavior `B-2026-05-31-active-landing-G03-validation`.

## Per-Gap Success Criteria

### GAP-001: Planned tasks can be selected for landing

- Current failure: a changed file bound to a `planned` task is selected for
  landing, so behavior verifiers run before the deterministic transition failure.
- Good behavior: Given an `active` task with a matching changed file, when
  `verify-landing` runs, then the command runs behavior verifiers and completes
  the selected task.
- Forbidden behavior: Given a `planned` task with a matching changed file and a
  failing verifier, when `verify-landing` runs, then the command rejects the
  target as non-active and does not run behavior verifiers.
- Files involved: `rust/task-registry-flow-cli/src/landing.rs`,
  `rust/task-registry-flow-cli/src/tests/landing_tests.rs`,
  `rust/task-registry-flow-cli/src/tests/mod.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_planned_targets_before_verifiers -- --nocapture`
- Data/schema/provenance: Planned rows keep `planned` status and receive no
  completion evidence.
- Runtime: `verify-landing` fails before verifier execution for planned rows.

### GAP-002: Runtime docs describe planned landing as valid

- Current failure: runtime documentation says landing maps changed files to
  planned or active task targets.
- Good behavior: Runtime docs describe `verify-landing` as active-task-only
  completion.
- Forbidden behavior: Runtime docs or landing errors keep saying planned targets
  are completion candidates.
- Files involved: `docs/runtime-schemas.md`,
  `rust/task-registry-flow-cli/src/landing.rs`,
  `rust/task-registry-flow-cli/src/tests/landing_tests.rs`.
- Positive test: `rg -n "active task target" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs`
- Negative test: `! rg -n "planned or active task target|planned/active task target" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs`
- Data/schema/provenance: No schema change.
- Runtime: N/A; docs and diagnostic text alignment.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_planned_targets_before_verifiers -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing -- --nocapture`
- `rg -n "active task target" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs`
- `! rg -n "planned or active task target|planned/active task target" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry report PLAN-2026-05-31-landing-active-only`
- `.codex/scripts/task-registry metrics`

## Walkthrough Evidence

- `PLAN_ACTIVATE docs/plans/landing-active-only-2026-05-31.md` output.
- Focused positive and negative landing test output.
- Positive and negative text-search output.
- Full validation output, or exact blocked command and reason.
- `TASK_VERIFY_LANDING` output.
- `TASK_REPORT PLAN-2026-05-31-landing-active-only` output.
- `TASK_METRICS` output.
- `VERIFY_CHAIN` output because activation and landing completion mutate
  registry receipts.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-landing-active-only"

[[behaviors]]
behavior_id = "B-2026-05-31-active-landing-G01-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Active tasks can land"
given = "An active task with a changed file bound to its target"
when = "verify-landing runs"
then = "behavior verifiers run and the selected task becomes completed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_completes_changed_file_tasks -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-active-landing-G01-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Planned tasks cannot land"
given = "A planned task with a changed file bound to its target and a failing verifier"
when = "verify-landing runs"
then = "the command rejects the target before verifier execution and leaves the task planned"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_planned_targets_before_verifiers -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml landing_rejects_planned_targets_before_verifiers -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-active-landing-G02-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Docs name active task targets"
given = "Runtime docs and landing diagnostics exist"
when = "workflow text is searched"
then = "active task target wording is present"
confirmation = "rg -n \"active task target\" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs"

[[behaviors.verifiers]]
type = "command"
command = "rg -n \"active task target\" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-active-landing-G02-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Docs and diagnostics do not name planned targets"
given = "Runtime docs and landing diagnostics exist"
when = "workflow text is searched for planned-target completion wording"
then = "no active runtime or diagnostic text names planned targets as landing candidates"
confirmation = "! rg -n \"planned or active task target|planned/active task target\" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n \"planned or active task target|planned/active task target\" docs/runtime-schemas.md rust/task-registry-flow-cli/src/landing.rs rust/task-registry-flow-cli/src/tests/landing_tests.rs"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-active-landing-G03-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Full active-only landing validation passes"
given = "Active-only landing selection is implemented"
when = "full project validation runs"
then = "source limit, registry validation, cargo tests, release readiness, receipt chain, report, and metrics pass"
confirmation = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry verify-chain --format json && .codex/scripts/task-registry report PLAN-2026-05-31-landing-active-only && .codex/scripts/task-registry metrics"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all && .codex/scripts/task-registry verify-chain --format json && .codex/scripts/task-registry report PLAN-2026-05-31-landing-active-only && .codex/scripts/task-registry metrics"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-active-landing-001"
status = "active"
title = "Restrict verify-landing to active tasks"
kind = "implementation"
reason = "Landing must reject planned task targets before running behavior verifiers."
acceptance_proof = "Behaviors B-2026-05-31-active-landing-G01-positive and B-2026-05-31-active-landing-G01-negative pass."
behavior_ids = ["B-2026-05-31-active-landing-G01-positive", "B-2026-05-31-active-landing-G01-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/landing.rs"
object = "landing task selector"
required_change = "Select only active task targets before verifier execution and update diagnostics."

[[tasks]]
task_id = "TASK-2026-05-31-active-landing-002"
status = "active"
title = "Cover planned-task landing rejection"
kind = "test"
reason = "The review gap requires a negative test proving planned targets fail before verifiers run."
acceptance_proof = "Behavior B-2026-05-31-active-landing-G01-negative passes."
behavior_ids = ["B-2026-05-31-active-landing-G01-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/landing_tests.rs"
object = "planned target rejection test"
required_change = "Add a test that a planned task with a failing verifier is rejected before verifier execution."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "landing failing-verifier fixture"
required_change = "Activate the task before expecting verifier failure."

[[tasks]]
task_id = "TASK-2026-05-31-active-landing-003"
status = "active"
title = "Sync active-only landing docs"
kind = "documentation"
reason = "Runtime docs and diagnostics must not describe planned tasks as landing candidates."
acceptance_proof = "Behaviors B-2026-05-31-active-landing-G02-positive and B-2026-05-31-active-landing-G02-negative pass."
behavior_ids = ["B-2026-05-31-active-landing-G02-positive", "B-2026-05-31-active-landing-G02-negative"]

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "Landing Completion Contract"
required_change = "Describe verify-landing as active-task-only completion."

[[tasks]]
task_id = "TASK-2026-05-31-active-landing-004"
status = "active"
title = "Run active-only landing validation"
kind = "validation"
reason = "The landing selector change touches governance completion authority and needs full release proof."
acceptance_proof = "Behavior B-2026-05-31-active-landing-G03-validation passes."
behavior_ids = ["B-2026-05-31-active-landing-G03-validation"]

[[tasks.targets]]
file = "docs/plans/landing-active-only-2026-05-31.md"
object = "validation and handoff evidence"
required_change = "Run the focused and full validation gates for active-only landing."
```
