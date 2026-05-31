# Verify-Chain Hardening & Release Policy Gap Closure Contract

## Approved Scope

Four gaps surfaced during `/verify` of the runtime.rs module split, `verify-chain` command, and `Diagnostic::warn` changes:

| ID | Gap | Verdict |
|----|-----|---------|
| GAP-001 | `verify-chain` silently ignores unknown arguments (`extra-arg`, `--unknown`) — should reject with usage | FAIL |
| GAP-002 | New `try_lock_exclusive()` in `append_event` breaks parallel test execution (`metrics_rejects_tampered_receipt_chain` panics) — `temp_root` nanosecond non-uniqueness | FAIL |
| GAP-003 | `verify-chain` text mode `format_chain_report` is blind to warnings — shows "receipt chain is intact" while JSON reveals 36 warnings | FAIL |
| GAP-004 | New Rust source files undeclared in `[release_source].required` → 6 `release-rust-source-undeclared` failures in `release-check required` | FAIL |

**In scope:** All four gaps. No deferrals.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/verify-chain-hardening-2026-05-31.md` — this contract
- [ ] `[MODIFY]` `REQUIREMENTS.toml` — `[release_source].required`: add 6 new source files

### Phase 1: GAP-001 (arg validation) + GAP-003 (warnings in text mode)
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `run_verify_chain`: replace silent `any()` arg parsing with strict `parse_verify_chain_args`
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `format_chain_report`: surface warning count when no failures

### Phase 2: GAP-002 (test contention)
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` — `temp_root`: add process-level atomic counter to guarantee uniqueness across parallel tests

### Phase 3: Validation
- [ ] `[VERIFY]` `.codex/scripts/task-registry validate`
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-chain` + probes
- [ ] `[VERIFY]` `.codex/scripts/task-registry release-check required`
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check`
- [ ] `[VERIFY]` `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=4`

## Per-Gap Success Criteria

### GAP-001: verify-chain silently ignores unknown arguments

- **Current failure:** `verify-chain extra-arg` → exit 0, "receipt chain is intact". `verify-chain --unknown` → same. Unknown args silently succeed.
- **Good behavior:** Given `verify-chain BAD_ARG`, the command returns exit 1 with the full usage string.
- **Forbidden behavior:** Unknown args must not cause a crash or silent success.
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** `verify-chain` (no args) → exit 0, "receipt chain is intact"
- **Negative test:** `verify-chain extra-arg` → exit 1, usage string
- **Domain/API/UI:** CLI arg parsing — match existing `parse_hook_format` pattern
- **Runtime:** `.codex/scripts/task-registry verify-chain BAD_ARG` → exit 1

### GAP-002: try_lock_exclusive breaks parallel test execution

- **Current failure:** `metrics_rejects_tampered_receipt_chain` panics under parallel test execution with "events file is locked by another process". Passes with `--test-threads=1`.
- **Good behavior:** Given parallel test execution (default test threads), all 95 tests pass with no lock contention.
- **Forbidden behavior:** Tests must not collide on temp directories or file locks.
- **Files involved:** `rust/task-registry-flow-cli/src/tests/mod.rs` (`temp_root`), `rust/task-registry-flow-cli/src/runtime.rs` (`append_event`)
- **Positive test:** `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml` → 95 passed, 0 failed
- **Negative test:** Run test suite 3 times with default parallelism → 0 failures each time
- **Domain/API/UI:** N/A — test infrastructure only
- **Runtime:** N/A — internal test harness

### GAP-003: verify-chain text mode is blind to warnings

- **Current failure:** `verify-chain` text mode outputs "receipt chain is intact" when the JSON mode reveals 36 `receipt-chain-unchained` warnings. Warnings are invisible in text output.
- **Good behavior:** Given a chain with only warnings (no breaks), text mode outputs "receipt chain is intact" followed by "N warning(s): ..." lines listing each warning.
- **Forbidden behavior:** Warnings must not cause a non-zero exit or be conflated with failures.
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** `verify-chain` (text mode, current events) → exit 0, summary includes warning count
- **Negative test:** `verify-chain` (text mode) with tampered chain → exit 1, shows both failures and warnings
- **Domain/API/UI:** CLI text output format — human-readable parity with JSON mode
- **Runtime:** `.codex/scripts/task-registry verify-chain` → exit 0, output includes "N warning(s)"

### GAP-004: New Rust source files undeclared in release policy

- **Current failure:** `release-check required` reports 6 `release-rust-source-undeclared` failures for `activation.rs`, `registry_io.rs`, `validation.rs`, `verify_chain.rs`, `state_transition_tests.rs`, `verify_chain_tests.rs`.
- **Good behavior:** Given a `release-check required` run, 0 undeclared source failures.
- **Forbidden behavior:** Files not checked into git or not part of the plugin must not be added.
- **Files involved:** `REQUIREMENTS.toml`
- **Positive test:** `.codex/scripts/task-registry release-check required` → 0 `release-rust-source-undeclared` failures
- **Negative test:** After adding declared files, removing one from disk → `release-check required` fails for that file
- **Domain/API/UI:** Release policy manifest — TOML array
- **Runtime:** `.codex/scripts/task-registry release-check required` → exit 0

## Validation Plan

**Focused:**
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain` && `.codex/scripts/task-registry verify-chain extra-arg` (expect exit 1)
- `.codex/scripts/task-registry release-check required`
- `.codex/scripts/task-registry source-limit check`
- `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=4`

**Full:**
- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

All files well under 1600 lines:
- `verify_chain.rs`: 238 lines → estimated +15 lines for changes → ~253
- `runtime.rs`: 463 lines → no change
- `schema.rs`: 652 lines → no change
- `tests/mod.rs` → check after change
- `REQUIREMENTS.toml`: 176 lines → +6 lines → ~182

## Walkthrough Evidence

1. `verify-chain extra-arg` → exit 1 with usage (GAP-001 fixed)
2. `verify-chain` text mode → shows "N warning(s)" (GAP-003 fixed)
3. `release-check required` → 0 `release-rust-source-undeclared` failures (GAP-004 fixed)
4. `cargo test -- --test-threads=4` → 95 passed (GAP-002 fixed)

## Task Manifest

```toml
schema_version = 2
plan_id = "verify-chain-hardening-2026-05-31"

[[behaviors]]
behavior_id = "B-001-verify-chain-rejects-unknown-args"
gap_id = "GAP-001"
polarity = "positive"
title = "verify-chain rejects unknown args with usage"
given = "the task-registry CLI"
when = "verify-chain is called with an unknown argument"
then = "the CLI prints usage and exits with code 1"
confirmation = ".codex/scripts/task-registry verify-chain BAD_ARG 2>&1; test $? -eq 1"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain BAD_ARG 2>&1; test $? -eq 1"
expected_exit = 0

[[behaviors]]
behavior_id = "B-001-verify-chain-accepts-valid-args-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "verify-chain still accepts valid args and no args"
given = "the task-registry CLI"
when = "verify-chain is called with no args, --repair, --format json, or their combination"
then = "the CLI processes normally (exit 0 or 1 depending on chain state, never usage for arg reasons)"
confirmation = ".codex/scripts/task-registry verify-chain 2>&1; test $? -eq 0"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain 2>&1; test $? -eq 0"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-parallel-tests-no-contention"
gap_id = "GAP-002"
polarity = "positive"
title = "parallel test execution completes with no lock contention failures"
given = "the full test suite with default parallelism"
when = "cargo test runs with 4 test threads"
then = "all 95 tests pass with 0 failures from lock contention"
confirmation = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=4"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=4"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-temp-root-unique-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "temp_root never produces colliding paths under rapid concurrent calls"
given = "two calls to temp_root with the same label at effectively the same time"
when = "paths are compared"
then = "the two paths differ"
confirmation = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=8"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=8"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-text-mode-surfaces-warnings"
gap_id = "GAP-003"
polarity = "positive"
title = "verify-chain text mode reports warning count when no failures"
given = "a receipt chain with unchained (warning) events but no breaks or tampers"
when = "verify-chain runs in text mode"
then = "output includes warning count and exits 0"
confirmation = ".codex/scripts/task-registry verify-chain 2>&1 | grep -q 'warning'"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain 2>&1 | grep -q 'warning'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-warnings-dont-break-exit-code-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "warnings do not cause non-zero exit"
given = "a receipt chain with warnings only (no breaks)"
when = "verify-chain runs in text mode"
then = "exit code is 0"
confirmation = ".codex/scripts/task-registry verify-chain 2>&1; test $? -eq 0"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain 2>&1; test $? -eq 0"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-release-check-zero-undeclared"
gap_id = "GAP-004"
polarity = "positive"
title = "release-check required has zero undeclared Rust sources"
given = "all new source files declared in REQUIREMENTS.toml"
when = "release-check required runs"
then = "zero release-rust-source-undeclared failures"
confirmation = ".codex/scripts/task-registry release-check required 2>&1 | grep 'release-rust-source-undeclared' | grep 'fail' | wc -l | xargs test 0 -eq"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check required 2>&1 | grep 'release-rust-source-undeclared' | grep 'fail' | wc -l | xargs test 0 -eq"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-release-check-fails-missing-source-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "release-check required fails when declared source is missing from disk"
given = "a declared source file removed from disk"
when = "release-check required runs"
then = "the corresponding release-file-present check fails"
confirmation = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- release_schema"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- release_schema"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-VC-001"
behavior_ids = ["B-004-release-check-zero-undeclared", "B-004-release-check-fails-missing-source-negative"]
title = "Add new Rust source files to REQUIREMENTS.toml release_source.required"
reason = "Six new source files from the runtime.rs module split are undeclared, causing release-check required failures"
acceptance_proof = "Behavior B-004: .codex/scripts/task-registry release-check required → 0 release-rust-source-undeclared failures"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "add six new source files: activation.rs, registry_io.rs, validation.rs, verify_chain.rs, state_transition_tests.rs, verify_chain_tests.rs"

[[tasks]]
task_id = "TASK-2026-05-31-VC-002"
behavior_ids = ["B-001-verify-chain-rejects-unknown-args", "B-001-verify-chain-accepts-valid-args-negative"]
title = "Add strict arg validation to verify-chain command"
reason = "Silent any() arg parsing in run_verify_chain allows unknown args to silently succeed instead of erroring with usage"
acceptance_proof = "Behavior B-001: verify-chain BAD_ARG → exit 1 with usage"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "run_verify_chain"
required_change = "replace silent any() arg parsing with strict validation that rejects unknown args"

[[tasks]]
task_id = "TASK-2026-05-31-VC-003"
behavior_ids = ["B-003-text-mode-surfaces-warnings", "B-003-warnings-dont-break-exit-code-negative"]
title = "Surface warnings in verify-chain text mode output"
reason = "format_chain_report only renders Fail diagnostics; Warn diagnostics are invisible in text mode while JSON mode shows them"
acceptance_proof = "Behavior B-003: verify-chain text output includes warning count"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "format_chain_report"
required_change = "surface warning count and warning details when no failures present"

[[tasks]]
task_id = "TASK-2026-05-31-VC-004"
behavior_ids = ["B-002-parallel-tests-no-contention", "B-002-temp-root-unique-negative"]
title = "Make temp_root directories reliably unique for parallel test execution"
reason = "Nanosecond-precision timestamp is insufficient for parallel test uniqueness, exposed by new try_lock_exclusive in append_event"
acceptance_proof = "Behavior B-002: cargo test -- --test-threads=4 → 95 passed, 0 lock contention failures"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "temp_root"
required_change = "add process-level atomic counter to guarantee unique directories across parallel tests"
```
