# Reviewer Report Closure Contract

## Approved Scope
Close `GP-002` and the remaining reviewer-output part of `GP-005` by adding a compact reviewer report command that separates governance proof from product correctness proof.

In scope:
- Add a `reviewer-report` CLI command.
- Summarize active plans, landed tasks, changed targets, local receipts, and deferred or blocked work.
- Include explicit proof-boundary language in the report output.
- Document the command in README and runtime schema docs.
- Update the gap pipeline with current evidence.

Out of scope:
- GitHub or pull request integration.
- Hosted dashboards or remote telemetry.
- Product/domain correctness evaluation.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/reviewer-report-2026-06-01.md` - `Task Manifest`: activate this contract before implementation edits.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation and landing keep validation green.

### Phase 1: Runtime command
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `CliCommand`: add typed `reviewer-report` command value.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/reviewer_report.rs` - `reviewer_report`: build compact text output from registry, metrics, changed files, and receipt counts.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: route `reviewer-report` and update usage text.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module`: include the new module.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `reviewer_report_tests_module`: register reviewer report tests.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs` - `reviewer_report_tests`: prove report output includes proof boundaries, active plan data, changed targets, and blocked/deferred work.

### Phase 2: Release policy and docs
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: require the new Rust source.
- [ ] `[MODIFY]` `README.md` - `reviewer handoff`: document `reviewer-report`.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `reviewer report`: document output intent and proof boundary.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-002` and `GP-005`: record the closed compact report evidence and leave PR integration as future work.

### Phase 3: Verification and handoff
- [ ] `[VERIFY]` `cargo test` - `reviewer_report`: focused reviewer report tests pass.
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check` - `line budget`: passes.
- [ ] `[VERIFY]` `.codex/scripts/task-registry validate` - `registry`: passes.

## Per-Gap Success Criteria
### GAP-001: Reviewer handoff lacks a compact product surface
- Current failure: maintainers must assemble `report`, `metrics`, `verify-chain`, changed targets, and blocked/deferred evidence manually.
- Good behavior: `reviewer-report` emits a pasteable governance summary with active plan count, landed task count, changed target count, receipt state, and blocked/deferred work.
- Forbidden behavior: the report omits blocked/deferred work or changed target evidence.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/reviewer_report.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/main.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs`, `REQUIREMENTS.toml`, `README.md`, `docs/runtime-schemas.md`, `docs/gap-pipeline.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture`
- Negative test: same focused test fixture asserts blocked/deferred task lines and changed target lines are present.
- Domain/API/UI: CLI command output only.
- Runtime: read-only command; no default receipt write.

### GAP-002: Reviewer output must not imply product correctness
- Current failure: README has proof-boundary language, but no reviewer output carries the same distinction.
- Good behavior: `reviewer-report` states that governance proof is not product correctness proof.
- Forbidden behavior: reviewer output claims green governance replaces product review or proves product correctness.
- Files involved: `rust/task-registry-flow-cli/src/reviewer_report.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `docs/gap-pipeline.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture`
- Negative test: typed `not_contains` verifier rejects `green governance means product correctness` in source and docs.
- Domain/API/UI: CLI command output only.
- Runtime: report is read-only and local-only.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit
Expected impact is moderate but split into a new Rust source. All touched files must remain under 1600 lines.

## Walkthrough Evidence
- Contract activation output.
- Focused reviewer-report test output.
- Source-limit, registry validation, receipt-chain, and release-readiness output.
- Task report and metrics output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-reviewer-report"

[[behaviors]]
behavior_id = "B-001-reviewer-report-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Reviewer report summarizes governance handoff evidence"
given = "A registry with active, completed, deferred, and blocked tasks"
when = "the reviewer report runs"
then = "it prints active plans, landed tasks, changed targets, receipt state, and blocked or deferred work"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-reviewer-report-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Reviewer report does not hide blocked or deferred work"
given = "A registry includes blocked and deferred task fixtures"
when = "the reviewer report is rendered"
then = "blocked and deferred tasks are visible in the output"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-proof-boundary-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Reviewer report states proof boundaries"
given = "Reviewer output is generated for handoff"
when = "the report is inspected"
then = "it states governance proof is not product correctness proof"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml reviewer_report -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-proof-boundary-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Reviewer report source avoids forbidden product proof overclaim"
given = "Reviewer report source and docs are present"
when = "forbidden proof language is checked"
then = "the source and docs do not claim green governance means product correctness"
confirmation = "typed not_contains verifiers for forbidden proof overclaim"

[[behaviors.verifiers]]
type = "not_contains"
path = "rust/task-registry-flow-cli/src/reviewer_report.rs"
needle = "green governance means product correctness"

[[behaviors.verifiers]]
type = "not_contains"
path = "docs/gap-pipeline.md"
needle = "green governance means product correctness"

[[tasks]]
task_id = "TASK-2026-06-01-reviewer-report-001"
behavior_ids = [
  "B-001-reviewer-report-positive",
  "B-002-reviewer-report-negative",
  "B-003-proof-boundary-positive",
  "B-004-proof-boundary-negative",
]
status = "planned"
title = "Add compact reviewer report"
kind = "implementation"
reason = "Maintainers need one local handoff surface for governance proof without overstating product correctness."
acceptance_proof = "Behaviors B-001 through B-004."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "CliCommand"
required_change = "Add typed reviewer-report command value."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reviewer_report.rs"
object = "reviewer_report"
required_change = "Render compact reviewer handoff evidence from registry and receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Route reviewer-report command and update usage."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "modules"
required_change = "Include reviewer_report module."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "reviewer_report_tests_module"
required_change = "Register reviewer report tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/reviewer_report_tests.rs"
object = "reviewer_report_tests"
required_change = "Add positive and negative reviewer report tests."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Declare reviewer_report.rs as a release source."

[[tasks.targets]]
file = "README.md"
object = "reviewer handoff"
required_change = "Document reviewer-report usage."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "Reviewer Report"
required_change = "Document reviewer report purpose and proof boundary."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-002 and GP-005"
required_change = "Record compact reviewer report closure evidence and remaining PR integration gap."
```
