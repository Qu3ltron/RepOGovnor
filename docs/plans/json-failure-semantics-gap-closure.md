# JSON Failure Semantics Gap Closure Contract

## Approved Scope

Approved production gaps:

- GAP-001: `status-check --format json` can emit failed diagnostics while exiting 0.
- GAP-002: `source-limit check --format json` discards per-file diagnostics on failure.
- GAP-003: `release-check ... --format json` routes failed JSON reports through text error rendering.
- GAP-004: global JSON CLI failures report `command = "usage"` and `receipt_recorded = false` even when the parsed command and receipt state differ.

Out of scope:

- New diagnostic schema versions, hosted reporting, remote telemetry, or compatibility adapters.
- Changes to successful text output beyond preserving existing behavior.

Runtime contract:

- Command-local diagnostic JSON failures must emit raw parseable JSON and exit nonzero.
- Global JSON failures must emit a `CommandReport` envelope with the actual parsed command and actual receipt state.
- Text failures may keep the existing `task-registry-flow error:` prefix.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/json-failure-semantics-gap-closure.md` - `closure_contract`: add approved scope, per-gap criteria, validation plan, and schema version 2 Task Manifest; acceptance proof: `PLAN_ACTIVATE docs/plans/json-failure-semantics-gap-closure.md`.
- [ ] `[MODIFY]` `docs/task-registry.toml` - `activated_plan`: activate this plan through the registry CLI; acceptance proof: `.codex/scripts/task-registry validate`.
- [ ] `[MODIFY]` `docs/task-registry/events.jsonl` - `activation_receipt`: record only CLI-governed activation/status receipts; acceptance proof: `.codex/scripts/task-registry metrics --format json`.

### Phase 1: Runtime JSON failure transport

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/reports.rs` - `runtime_failure_transport`: add typed helpers that identify raw JSON diagnostic failures without text prefixing; acceptance proof: behaviors `B-2026-05-31-json-G03-positive` and `B-2026-05-31-json-G03-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cli.rs` - `actual_failure_envelope`: render global JSON failures with the parsed command and computed receipt state; acceptance proof: behaviors `B-2026-05-31-json-G04-positive` and `B-2026-05-31-json-G04-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `status_check_json_failure`: make failed `status-check --format json` return nonzero while preserving the raw diagnostic report; acceptance proof: behaviors `B-2026-05-31-json-G01-positive` and `B-2026-05-31-json-G01-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/source_limit.rs` - `source_limit_json_failure`: return the full `CheckReport` JSON on failing JSON checks; acceptance proof: behaviors `B-2026-05-31-json-G02-positive` and `B-2026-05-31-json-G02-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `release_json_failure`: return raw diagnostic JSON on failing JSON checks without top-level text error wrapping; acceptance proof: behaviors `B-2026-05-31-json-G03-positive` and `B-2026-05-31-json-G03-negative`.

### Phase 2: Negative migration tests

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `status_json_failure_tests`: add missing/symlink native-skill JSON failure assertions; acceptance proof: behavior `B-2026-05-31-json-G01-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `source_limit_json_failure_tests`: assert over-limit JSON keeps failed file diagnostics; acceptance proof: behavior `B-2026-05-31-json-G02-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `release_json_failure_tests`: assert failed release JSON is raw parseable JSON; acceptance proof: behavior `B-2026-05-31-json-G03-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `cli_failure_envelope_tests`: assert failed global JSON reports actual command and receipt state; acceptance proof: behavior `B-2026-05-31-json-G04-negative`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `json_failure_migration_suite`: add shell-level parse checks for failing release/source JSON reports; acceptance proof: behavior `B-2026-05-31-json-failure-validation`.

### Phase 3: Documentation and handoff

- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `json_failure_contract`: document raw diagnostic JSON for command-local failures and command envelopes for global failures; acceptance proof: behavior `B-2026-05-31-json-failure-validation`.
- [ ] `[MODIFY]` `README.md` - `user_json_failure_examples`: align documented command examples with real exit semantics; acceptance proof: behavior `B-2026-05-31-json-failure-validation`.
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check` - `source_budget`: prove touched files remain at or below 1600 lines; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml` - `rust_behavior_suite`: prove positive and negative JSON failure behavior; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `bash scripts/test-release-readiness.sh all` - `migration_suite`: prove shell-level JSON failure migration behavior; acceptance proof: command exits 0.

## Per-Gap Success Criteria

### GAP-001: `status-check --format json` exits honestly

- Current failure: Missing or symlinked `.agents/skills/task-registry-flow` can emit failed diagnostics while returning `Ok(report.to_json())`.
- Good behavior: Given missing or symlinked native skill projection when `status-check --format json` runs, then output is parseable JSON with `check_id = "native-skill"` and exit is nonzero.
- Forbidden behavior: Any JSON report containing failed checks exits 0.
- Files involved: `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_failure_exits_nonzero`.
- Data/schema/provenance: JSON remains a `CheckReport` with `surface = "status"`.
- Runtime: Process exit status matches diagnostic failure state.

### GAP-002: `source-limit check --format json` preserves diagnostics

- Current failure: Over-limit JSON mode returns only `source file limit exceeded`.
- Good behavior: Given an over-limit file when `source-limit check --format json` runs, then output is parseable `CheckReport` JSON with the violating path and exit is nonzero.
- Forbidden behavior: Failure output loses `check_id`, `path`, `expected`, `actual`, or `remediation`.
- Files involved: `rust/task-registry-flow-cli/src/source_limit.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `scripts/test-release-readiness.sh`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_success_exits_zero`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_failure_preserves_diagnostics`.
- Data/schema/provenance: JSON uses the existing `CheckReport` shape.
- Runtime: Machine-readable failure diagnostics survive nonzero exit.

### GAP-003: `release-check ... --format json` stays valid on failure

- Current failure: Failing release JSON returns serialized JSON as `Err`, and top-level text rendering prefixes it.
- Good behavior: Given a release-source failure when `release-check all --format json` runs, then output is raw parseable `CheckReport` JSON with exit nonzero and no text prefix.
- Forbidden behavior: JSON failure output starts with `task-registry-flow error:` or any non-JSON text.
- Files involved: `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `scripts/test-release-readiness.sh`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_success_exits_zero`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_failure_preserves_report`.
- Data/schema/provenance: JSON uses the existing `CheckReport` shape and `release-file-present` check ids.
- Runtime: Command-local JSON failure bypasses human error prefixing.

### GAP-004: Global JSON failures report actual command and receipt state

- Current failure: Failed global JSON envelopes hard-code `command = "usage"` and `receipt_recorded = false`.
- Good behavior: Given `--record-receipt --format json validate extra`, when the command fails, then the envelope reports `command = "validate"` and `receipt_recorded = true`.
- Forbidden behavior: A known parsed command failure is reported as `usage`, or receipt state disagrees with whether a receipt was written.
- Files involved: `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/reports.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_reports_actual_command`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_reports_receipt_state`.
- Data/schema/provenance: `CommandReport` remains schema version 2.
- Runtime: Failure envelope matches actual parsed command and receipt policy.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-json-failure-semantics`

## Walkthrough Evidence

Capture:

- `PLAN_ACTIVATE docs/plans/json-failure-semantics-gap-closure.md` output.
- Focused positive and negative test outputs.
- Full validation command outputs.
- `.codex/scripts/task-registry report PLAN-2026-05-31-json-failure-semantics`.
- `.codex/scripts/task-registry metrics --format json`.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-json-failure-semantics"

[[behaviors]]
behavior_id = "B-2026-05-31-json-G01-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Status JSON success exits zero"
given = "A workspace with a native task-registry-flow skill directory"
when = "status-check --format json runs"
then = "The status report has no failures and exits zero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G01-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Status JSON failure exits nonzero"
given = "A workspace with a missing or symlinked task-registry-flow skill"
when = "status-check --format json runs"
then = "The status report is parseable JSON, contains native-skill failure, and exits nonzero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G02-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Source-limit JSON success exits zero"
given = "A workspace within the source file line budget"
when = "source-limit check --format json runs"
then = "The source-limit report has no failures and exits zero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G02-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Source-limit JSON failure preserves diagnostics"
given = "A workspace with an over-limit source file"
when = "source-limit check --format json runs"
then = "The source-limit report is parseable JSON, includes the violating path, and exits nonzero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_failure_preserves_diagnostics"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml source_limit_json_failure_preserves_diagnostics"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G03-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Release JSON success exits zero"
given = "A release fixture satisfying required release checks"
when = "release-check all --format json runs"
then = "The release report has no failures and exits zero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G03-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Release JSON failure preserves report"
given = "A release fixture missing a required file"
when = "release-check all --format json runs"
then = "The release report is raw parseable JSON with release-file-present failure and exits nonzero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_failure_preserves_report"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_check_json_failure_preserves_report"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G04-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "CLI JSON failure reports actual command"
given = "A known command fails under global JSON output"
when = "The CLI renders the failure envelope"
then = "The envelope command is the parsed command, not usage"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_reports_actual_command"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_reports_actual_command"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-G04-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "CLI JSON failure reports receipt state"
given = "A failing known command is run with record-receipt enabled"
when = "The CLI renders the failure envelope"
then = "The envelope reports receipt_recorded true"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_reports_receipt_state"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_error_reports_receipt_state"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-json-failure-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Full JSON failure validation passes"
given = "All JSON failure semantics are implemented"
when = "Full validation runs"
then = "Rust tests, release readiness, source limit, and registry validation pass"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-json-failure-001"
title = "Make status-check JSON failures exit nonzero"
status = "planned"
kind = "diagnostics"
reason = "Failed status diagnostic JSON can currently pass with exit 0."
acceptance_proof = "Behaviors B-2026-05-31-json-G01-positive and B-2026-05-31-json-G01-negative pass."
behavior_ids = ["B-2026-05-31-json-G01-positive", "B-2026-05-31-json-G01-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "status_check_command"
required_change = "Return nonzero on failed JSON reports while preserving JSON."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "status_check_api"
required_change = "Own JSON-aware status-check behavior."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "status_check_json_tests"
required_change = "Add positive and negative JSON exit assertions."

[[tasks]]
task_id = "TASK-2026-05-31-json-failure-002"
title = "Preserve source-limit JSON diagnostics on failure"
status = "planned"
kind = "diagnostics"
reason = "Source-limit JSON failure currently loses per-file diagnostics."
acceptance_proof = "Behaviors B-2026-05-31-json-G02-positive and B-2026-05-31-json-G02-negative pass."
behavior_ids = ["B-2026-05-31-json-G02-positive", "B-2026-05-31-json-G02-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/source_limit.rs"
object = "json_failure_report"
required_change = "Return full CheckReport JSON and nonzero status on failed JSON checks."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "source_limit_json_tests"
required_change = "Assert over-limit JSON includes failed path diagnostics."

[[tasks]]
task_id = "TASK-2026-05-31-json-failure-003"
title = "Preserve release-check JSON reports on failure"
status = "planned"
kind = "diagnostics"
reason = "Release-check JSON failure can be prefixed as human text."
acceptance_proof = "Behaviors B-2026-05-31-json-G03-positive and B-2026-05-31-json-G03-negative pass."
behavior_ids = ["B-2026-05-31-json-G03-positive", "B-2026-05-31-json-G03-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "json_failure_report"
required_change = "Return full CheckReport JSON and nonzero status on failed JSON checks."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "raw_json_error_rendering"
required_change = "Render raw JSON diagnostic failures without human prefix."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "release_check_json_tests"
required_change = "Assert failing release JSON is parseable raw JSON."

[[tasks]]
task_id = "TASK-2026-05-31-json-failure-004"
title = "Report actual CLI command and receipt state"
status = "planned"
kind = "schema"
reason = "Global JSON failure envelope hard-codes usage and receipt false."
acceptance_proof = "Behaviors B-2026-05-31-json-G04-positive and B-2026-05-31-json-G04-negative pass."
behavior_ids = ["B-2026-05-31-json-G04-positive", "B-2026-05-31-json-G04-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "failure_envelope"
required_change = "Render parsed command and computed receipt state on failures."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reports.rs"
object = "failure_report_helpers"
required_change = "Support command-specific failure envelopes."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "cli_json_error_tests"
required_change = "Assert actual command and receipt state in JSON failures."

[[tasks]]
task_id = "TASK-2026-05-31-json-failure-005"
title = "Add shell migration coverage and docs"
status = "planned"
kind = "test"
reason = "Release shell migration must prove JSON failures stay machine-readable."
acceptance_proof = "Behavior B-2026-05-31-json-failure-validation passes."
behavior_ids = ["B-2026-05-31-json-failure-validation"]

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "json_failure_migration_suite"
required_change = "Parse failing release/source JSON and assert nonzero exits."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "json_failure_contract"
required_change = "Document command-local raw JSON failures and global failure envelopes."

[[tasks.targets]]
file = "README.md"
object = "json_failure_examples"
required_change = "Keep user examples aligned with real JSON semantics."
```
