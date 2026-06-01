# Typed Runtime Surfaces Gap Closure Contract

## Approved Scope
Ensure runtime-facing command, hook format, receipt/event, and failure-code surfaces are typed.

In scope:
- Keep `CliCommand` as the only command discriminator used by CLI reports and receipts.
- Keep `HookFormat` as the only hook payload format discriminator.
- Replace stringly receipt/event subject kinds and diagnostic report surfaces with typed enums.
- Add typed failure codes to command failures and runtime failures.
- Add regression coverage proving unknown command/hook/surface/failure values fail closed.

Out of scope: changing public command names, adding compatibility aliases, changing task manifest syntax, or changing release version.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/typed-runtime-surfaces-2026-06-01.md` - `Task Manifest`: activate this contract before implementation.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation, landing, and archive leave typed registry validation green.

### Phase 1: Typed runtime model
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `runtime schema`: add typed schema version, runtime subject kind, report surface, and failure code enums; wire them into receipt events, diagnostics, check reports, and command reports.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/reports.rs` - `RuntimeFailure`: replace untyped text/json failures with typed failure codes and typed JSON payload handling.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cli.rs` - `error rendering`: emit typed failure codes in global JSON failures and preserve typed command enum routing.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatch`: preserve typed `CliCommand` and `HookFormat` parsing with no string aliases.

### Phase 2: Typed producers and tests
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/source_limit.rs` - `diagnostic output`: use typed report surfaces.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/status_checks.rs` - `diagnostic output`: use typed report surfaces.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `diagnostic output`: use typed report surfaces.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` - `diagnostic output`: use typed receipt-chain surface and typed failures.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verifiers.rs` - `verifier subjects`: use typed verifier target subject kind.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs` - `typed runtime regressions`: prove typed command, hook format, receipt/event schema, report surface, and failure-code behavior.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `test module list`: include the new focused test file.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: declare the new Rust test source.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `runtime schemas`: document typed failure codes and typed surfaces.

## Per-Gap Success Criteria
### GAP-001: Typed command enum only
- Current failure: command routing is mostly typed, but completion needs verifier proof that unknown command strings cannot enter reports or receipts as commands.
- Good behavior: command reports and receipts serialize only `CliCommand` variants.
- Forbidden behavior: unknown command text is accepted as a command discriminator.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_command_enum_round_trips -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_command_enum_rejects_unknown_values -- --nocapture`
- Domain/API/UI: command strings stay the current public CLI names.
- Runtime: invalid command values fail before command execution.

### GAP-002: Typed hook format enum only
- Current failure: hook format enum exists, but completion needs verifier proof that unknown format strings fail closed.
- Good behavior: hook parsing accepts only `antigravity`, `codex`, `cursor`, and `claude`.
- Forbidden behavior: unknown hook format text reaches payload validation.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_hook_format_enum_round_trips -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_hook_format_rejects_unknown_values -- --nocapture`
- Domain/API/UI: hook format names stay unchanged.
- Runtime: unknown formats return an error before reading hook JSON.

### GAP-003: Typed receipt/event schema
- Current failure: receipt events are structured, but `RuntimeSubject.kind`, diagnostic `surface`, and report `surface` are free strings.
- Good behavior: receipt/event schema uses typed schema version, typed subject kind, typed command, typed outcome, typed diagnostics, and typed report surfaces.
- Forbidden behavior: unknown subject kinds or report surfaces deserialize as valid runtime data.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/source_limit.rs`, `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/verify_chain.rs`, `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_round_trips -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_rejects_unknown_subject_kind -- --nocapture`
- Domain/API/UI: JSON wire values remain stable strings and numeric schema versions.
- Runtime: receipt chain verification still accepts current v2 receipts and rejects malformed values.

### GAP-004: Typed failure codes
- Current failure: runtime failures carry only text or raw JSON payloads, so machine consumers cannot rely on a typed failure discriminator.
- Good behavior: every runtime failure has a `FailureCode`, and global JSON failures serialize that code.
- Forbidden behavior: a command failure JSON payload lacks a typed `failure_code`.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/reports.rs`, `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_emits_in_json_report -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_rejects_unknown_values -- --nocapture`
- Domain/API/UI: human text can still render summaries; JSON owns the typed failure code.
- Runtime: command failures remain nonzero.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_ -- --nocapture`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-release-readiness.sh all`
- `bash scripts/release-audit.sh`

## Source File Limit
Expected impact is a small schema expansion plus one focused test module because `rust/task-registry-flow-cli/src/tests/mod.rs` is near the 1600-line limit. Run `.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence
- Contract activation output.
- Focused typed runtime test output.
- Full Rust, clippy, release readiness, and release audit output.
- `verify-landing` completion output for this plan.
- `verify-chain --format json` output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-typed-runtime-surfaces"

[[behaviors]]
behavior_id = "B-001-command-enum-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Command enum round trips"
given = "The public command vocabulary"
when = "command values serialize and parse"
then = "only CliCommand variants are represented"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_command_enum_round_trips -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_command_enum_round_trips -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-command-enum-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Unknown command values fail closed"
given = "An unknown command discriminator"
when = "the runtime parses it"
then = "it is rejected before execution"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_command_enum_rejects_unknown_values -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_command_enum_rejects_unknown_values -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-hook-format-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Hook format enum round trips"
given = "The public hook format vocabulary"
when = "hook format values serialize and parse"
then = "only HookFormat variants are represented"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_hook_format_enum_round_trips -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_hook_format_enum_round_trips -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-hook-format-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Unknown hook formats fail closed"
given = "An unknown hook format discriminator"
when = "the runtime parses it"
then = "it is rejected before hook payload validation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_hook_format_rejects_unknown_values -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_hook_format_rejects_unknown_values -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-receipt-schema-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Receipt event schema is typed"
given = "A schema version 2 receipt event"
when = "it serializes and deserializes"
then = "typed command, outcome, subject kind, diagnostics, and surfaces round trip"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_round_trips -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_round_trips -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-receipt-schema-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Unknown receipt subject kind fails closed"
given = "A receipt event with an unknown subject kind"
when = "it deserializes"
then = "serde rejects the value"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_rejects_unknown_subject_kind -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_rejects_unknown_subject_kind -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007-failure-code-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "JSON failures include typed failure code"
given = "A command failure rendered as JSON"
when = "the report is serialized"
then = "it includes a typed failure_code value"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_emits_in_json_report -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_emits_in_json_report -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-008-failure-code-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Unknown failure codes fail closed"
given = "A JSON report with an unknown failure_code"
when = "it deserializes as a command report"
then = "serde rejects the value"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_rejects_unknown_values -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_rejects_unknown_values -- --nocapture"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-typed-runtime-surfaces-001"
behavior_ids = [
  "B-001-command-enum-positive",
  "B-002-command-enum-negative",
  "B-003-hook-format-positive",
  "B-004-hook-format-negative",
  "B-005-receipt-schema-positive",
  "B-006-receipt-schema-negative",
  "B-007-failure-code-positive",
  "B-008-failure-code-negative",
]
status = "planned"
title = "Enforce typed runtime command, hook, receipt, and failure surfaces"
kind = "implementation"
reason = "Runtime machine interfaces must use typed discriminators instead of open strings."
acceptance_proof = "Behaviors B-001 through B-008."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "typed runtime schema"
required_change = "Add typed schema version, subject kind, report surface, and failure code fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reports.rs"
object = "typed RuntimeFailure"
required_change = "Carry typed failure codes through runtime errors and JSON payloads."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "failure rendering"
required_change = "Emit typed failure_code in command reports."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "typed dispatch"
required_change = "Preserve typed command and hook parsing without string aliases."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/source_limit.rs"
object = "typed report surfaces"
required_change = "Use typed report surface enum values."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "typed report surfaces"
required_change = "Use typed report surface enum values."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "typed report surfaces"
required_change = "Use typed report surface enum values."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs"
object = "typed runtime regressions"
required_change = "Add positive and negative tests for typed command, hook, receipt, and failure surfaces."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "typed report surfaces and failures"
required_change = "Use typed receipt-chain surface and typed runtime failures."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verifiers.rs"
object = "typed verifier subject"
required_change = "Use typed verifier-target runtime subject kind."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "test module list"
required_change = "Include typed_runtime_surface_tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/status_check_tests.rs"
object = "status check typed surface tests"
required_change = "Update status-check tests for typed report surfaces and typed JSON runtime failures."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Declare new typed runtime test source."

[[tasks.targets]]
file = "docs/plans/typed-runtime-surfaces-2026-06-01.md"
object = "closure contract"
required_change = "Track approved scope, typed behavior verifiers, targets, and validation evidence."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "runtime schema documentation"
required_change = "Document typed failure codes, typed subject kinds, and typed report surfaces."

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "task registry activation and landing"
required_change = "Record this plan state through task-registry CLI only."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "task registry receipts"
required_change = "Append activation and landing receipts through task-registry CLI only."

[[tasks.targets]]
file = "docs/task-registry/archive/"
object = "completed task archive"
required_change = "Archive completed task rows after landing."
```
