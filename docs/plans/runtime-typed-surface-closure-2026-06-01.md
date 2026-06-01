# Runtime Typed Surface Closure Contract

## Approved Scope
Ensure the runtime surfaces requested by the active goal are typed end to end:
typed command enum, typed hook format enum, typed receipt/event schema, and
typed failure codes.

In scope:
- Remove raw fallback failure-code literals from JSON report fallback paths.
- Make receipt/event deserialization reject non-v2 runtime event schema versions.
- Add positive and negative tests proving command, hook format, receipt/event,
  and failure-code typing.

Out of scope: changing registry archive schema, release check schema, verifier
command strings, or shell command parsing.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/runtime-typed-surface-closure-2026-06-01.md` - `Task Manifest`: activate this contract before implementation edits.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation, landing, and archive keep validation green.

### Phase 1: Runtime typed surface hardening
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `ReceiptEvent.schema_version`: enforce a typed v2-only receipt/event schema.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cli.rs` - `JSON serialization fallback`: derive fallback failure code from `FailureCode::Serialization`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs` - `typed runtime surface tests`: prove typed command, hook format, receipt/event schema, and failure-code behavior.

## Per-Gap Success Criteria
### GAP-001: Receipt/event schema accepts a v1 runtime event shape
- Current failure: `ReceiptEvent` uses the generic schema-version enum, so a v1 value can deserialize if it has the current event fields.
- Good behavior: receipt/event JSON must deserialize only with schema version 2.
- Forbidden behavior: a schema version 1 runtime event with otherwise valid fields deserializes as a current receipt.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_round_trips -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_rejects_v1_and_unknown_subjects -- --nocapture`
- Domain/API/UI: runtime JSON receipt schema only.
- Runtime: receipt-chain and metrics behavior remain typed and validating.

### GAP-002: JSON report fallback has raw failure-code literals
- Current failure: JSON fallback formatting embeds `failure_code":"serialization"` as a raw string.
- Good behavior: fallback JSON obtains the code through `FailureCode::Serialization`.
- Forbidden behavior: fallback output can drift from the typed failure-code enum spelling.
- Files involved: `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_emits_in_json_report -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_rejects_unknown_values -- --nocapture`
- Domain/API/UI: runtime JSON command report only.
- Runtime: text errors and normal JSON report serialization remain unchanged.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_runtime_surface_tests -- --nocapture`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit
Expected impact is small edits in files currently under the 1600-line limit.

## Walkthrough Evidence
- Contract activation output.
- Focused typed runtime surface test output.
- Full Rust test and release readiness output.
- Task report, metrics, source-limit, validation, and receipt-chain output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-runtime-typed-surface-closure"

[[behaviors]]
behavior_id = "B-001-receipt-schema-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Receipt event schema v2 round trips through typed runtime fields"
given = "A receipt event created by the runtime schema constructor"
when = "it serializes and deserializes through JSON"
then = "command, outcome, subject kind, and report surface remain typed enum values"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_round_trips -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_round_trips -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-receipt-schema-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Receipt event schema rejects v1 and unknown subject kinds"
given = "A runtime event JSON payload with a forbidden schema version or subject kind"
when = "it deserializes as a receipt event"
then = "deserialization fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_rejects_v1_and_unknown_subjects -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_receipt_event_schema_rejects_v1_and_unknown_subjects -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-failure-code-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "JSON command reports emit typed failure codes"
given = "A runtime command failure report"
when = "it serializes to JSON"
then = "failure_code deserializes as the typed FailureCode enum"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_emits_in_json_report -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_emits_in_json_report -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-failure-code-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "JSON command reports reject unknown failure codes"
given = "A command report JSON payload with an unknown failure code"
when = "it deserializes as a command report"
then = "deserialization fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_rejects_unknown_values -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_failure_code_rejects_unknown_values -- --nocapture"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-runtime-typed-surface-closure-001"
behavior_ids = [
  "B-001-receipt-schema-positive",
  "B-002-receipt-schema-negative",
  "B-003-failure-code-positive",
  "B-004-failure-code-negative",
]
status = "planned"
title = "Close runtime typed surface gaps"
kind = "implementation"
reason = "The active goal requires typed command, hook format, receipt/event schema, and failure-code surfaces only."
acceptance_proof = "Behaviors B-001 through B-004."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "ReceiptEvent schema version typing"
required_change = "Reject non-v2 receipt/event schema versions during deserialization."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "JSON fallback failure code"
required_change = "Use FailureCode::Serialization instead of raw fallback failure-code literals."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/typed_runtime_surface_tests.rs"
object = "typed runtime surface tests"
required_change = "Prove typed command, hook format, receipt/event schema, and failure-code behavior."

[[tasks.targets]]
file = "docs/plans/runtime-typed-surface-closure-2026-06-01.md"
object = "closure contract"
required_change = "Track approved scope, behavior verifiers, and validation evidence."

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
object = "completed task archives"
required_change = "Archive completed task rows after landing."
```
