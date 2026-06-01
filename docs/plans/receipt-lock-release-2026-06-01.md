# Receipt Lock Release Gap Closure Contract

## Approved Scope
Close the CI/Nix flake where sequential receipt appends can see `events file is locked by another process` in metrics tests.

In scope:
- Explicitly release the event-file lock after durable receipt append.
- Verify the focused metrics test and Nix package check.

Out of scope: changing receipt schema, lock error text, or receipt-chain hash semantics.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/receipt-lock-release-2026-06-01.md` - `Task Manifest`: activate this contract before implementation.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation, landing, and archive keep validation green.

### Phase 1: Lock release
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `append_event`: unlock the event file after successful sync.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli/src/tests/metrics_tests.rs` - `metrics_validates_chained_receipts`: prove sequential appends no longer trip the receipt lock.

## Per-Gap Success Criteria
### GAP-001: Sequential receipt append lock remains held in CI/Nix
- Current failure: Nix checkPhase intermittently fails `metrics_validates_chained_receipts` with `events file is locked by another process`.
- Good behavior: after `append_event` syncs the receipt line, the file lock is explicitly released before returning.
- Forbidden behavior: a second sequential append in the same test process fails because the previous append still holds the lock.
- Files involved: `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/metrics_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts -- --nocapture`
- Negative test: `nix build .#task-registry-flow --no-link`
- Domain/API/UI: N/A; runtime receipt behavior only.
- Runtime: receipt append remains durable and chained.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts -- --nocapture`
- `nix build .#task-registry-flow --no-link`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-release-readiness.sh all`
- `bash scripts/release-audit.sh`

## Source File Limit
Expected impact is a small change in `runtime.rs`, below the 1600-line limit.

## Walkthrough Evidence
- Contract activation output.
- Focused metrics test output.
- Nix package build output.
- Release readiness and GitHub CI output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-receipt-lock-release"

[[behaviors]]
behavior_id = "B-001-receipt-lock-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Sequential receipt appends release locks"
given = "Two chained receipt appends in one test process"
when = "the first append syncs and returns"
then = "the second append can acquire the events file lock"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-nix-lock-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Nix package check rejects receipt lock regressions"
given = "Nix checkPhase runs the Rust unit suite"
when = "receipt metrics tests run under the Nix builder"
then = "no receipt lock regression fails the package build"
confirmation = "nix build .#task-registry-flow --no-link"

[[behaviors.verifiers]]
type = "command"
command = "nix build .#task-registry-flow --no-link"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-receipt-lock-release-001"
behavior_ids = [
  "B-001-receipt-lock-positive",
  "B-002-nix-lock-negative",
]
status = "planned"
title = "Release receipt file locks after durable append"
kind = "implementation"
reason = "CI/Nix package check exposed sequential receipt append lock retention."
acceptance_proof = "Behaviors B-001 and B-002."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "append_event lock release"
required_change = "Explicitly unlock the events file after sync succeeds."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/metrics_tests.rs"
object = "receipt metrics lock regression"
required_change = "Use existing focused metrics test as behavior proof."

[[tasks.targets]]
file = "docs/plans/receipt-lock-release-2026-06-01.md"
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
