# Verify-Chain Hardening Batch 2 — Gap Closure Contract

## Approved Scope

Five hardening opportunities surfaced during review of `verify_chain.rs` after the initial gap-closure pass:

| ID | Gap | Severity |
|----|-----|----------|
| H-001 | `repair_chain` opens events file with `truncate(true)` before acquiring the lock or writing the temp file — data-loss hazard if lock, write, or sync fails | HIGH |
| H-002 | `parse_verify_chain_args` silently accepts duplicate flags (`--repair --repair`, `--format json --format json`) | MEDIUM |
| H-003 | Error messages leak Rust `Option` debug formatting (`Some("garbage")`, `None`) to user output | LOW |
| H-004 | `format_chain_report` ignores `pass` diagnostics — after `--repair`, users see "receipt chain is intact" but never learn what lines were fixed | MEDIUM |
| H-005 | Orphaned temp file (`events.jsonl.tmp`) left behind when `sync_all` fails after a successful `fs::write` | LOW |

**In scope:** All five. No deferrals.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/verify-chain-hardening-batch-2-2026-05-31.md` — this contract

### Phase 1: H-001 (truncation before lock)
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `repair_chain`: change `truncate(true)` to `truncate(false)` so the events file is not truncated before lock acquisition and atomic rename

### Phase 2: H-002 (duplicate flags) + H-003 (Option debug in errors)
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `parse_verify_chain_args`: detect duplicate `--repair` and `--format` flags; use user-friendly error messages without `Option` debug formatting

### Phase 3: H-004 (pass diagnostics) + H-005 (orphaned tmp cleanup)
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `format_chain_report`: render pass diagnostics when present so repair output surfaces what was fixed
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `repair_chain`: clean up tmp file on sync failure

### Phase 4: Validation
- [ ] `[VERIFY]` `.codex/scripts/task-registry validate`
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-chain` + probes for H-001 through H-005
- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check`
- [ ] `[VERIFY]` `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=4`

## Per-Gap Success Criteria

### H-001: repair_chain truncates events file before safe write

- **Current failure:** `repair_chain` opens the events file with `OpenOptions::new().create(true).write(true).truncate(true)` on line 221. This truncates the file to zero bytes *before* acquiring `try_lock_exclusive()` and *before* successfully writing and syncing the temp file. If the lock, write, or sync fails, the events file is already empty and the original data is lost.
- **Good behavior:** Given repair is requested, the events file is not truncated until the atomic rename of the temp file overwrites it. If any step before the rename fails, the original events file is untouched.
- **Forbidden behavior:** The events file must never be truncated to zero bytes before the atomic rename succeeds.
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** `verify-chain --repair` on a tampered chain succeeds and produces an intact chain
- **Negative test:** Simulate a lock-contention failure during repair — the events file must retain its original content
- **Domain/API/UI:** N/A — internal file-safety invariant
- **Runtime:** `.codex/scripts/task-registry verify-chain --repair` → exit 0, events file intact

### H-002: parse_verify_chain_args silently accepts duplicate flags

- **Current failure:** `verify-chain --repair --repair` (exit 0, silent), `verify-chain --format json --format json` (exit 0, silent). No other CLI subcommand in this crate allows duplicate flags — `--format` is rejected after the first positional by `parse_global_options`.
- **Good behavior:** Given a duplicate `--repair` or `--format`, the parser returns an error: `duplicate flag: --repair`.
- **Forbidden behavior:** Duplicate flags must not silently succeed or cause a crash.
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** `verify-chain --repair` (single occurrence) → succeeds
- **Negative test:** `verify-chain --repair --repair` → exit 1, error mentions duplicate
- **Domain/API/UI:** CLI arg parsing — consistency with crate conventions
- **Runtime:** `.codex/scripts/task-registry verify-chain --repair --repair` → exit 1

### H-003: Error messages leak Rust Option debug formatting

- **Current failure:** `verify-chain --format garbage` outputs `got: Some("garbage")`. `verify-chain --format` (no value) outputs `got: None`. The `Option` debug representation is not user-facing language.
- **Good behavior:** Given an invalid `--format` value, the error says `got: "garbage"`. Given a missing value, the error says `got: (nothing)`.
- **Forbidden behavior:** Error messages must not display Rust internal type representations.
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** `verify-chain --format json` → succeeds
- **Negative test:** `verify-chain --format garbage` → exit 1, error message contains `"garbage"` not `Some("garbage")`
- **Domain/API/UI:** CLI error message quality
- **Runtime:** `.codex/scripts/task-registry verify-chain --format garbage 2>&1` → no `Some(` in output

### H-004: format_chain_report ignores pass diagnostics

- **Current failure:** After `verify-chain --repair`, the repair function returns a `CheckReport` populated with `Diagnostic::pass("receipt-chain-repaired", ...)` entries for each fixed line. `format_chain_report` only iterates `CheckStatus::Fail` and `CheckStatus::Warn` diagnostics — pass entries are silently dropped. The user sees "receipt chain is intact" with no indication of what was repaired.
- **Good behavior:** Given a repair that fixed 2 lines, text mode output includes "2 repair(s):" followed by per-line details.
- **Forbidden behavior:** Pass diagnostics must not be conflated with failures (must not cause non-zero exit).
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** `verify-chain --repair` on a tampered chain → output includes repair count and per-line details
- **Negative test:** `verify-chain` (no repair) on an intact chain → output still shows "receipt chain is intact" without spurious repair lines
- **Domain/API/UI:** CLI text output — human-readable parity with diagnostics data
- **Runtime:** `.codex/scripts/task-registry verify-chain --repair` (with tampered chain) → exit 0, output includes repair lines

### H-005: Orphaned temp file on sync failure

- **Current failure:** In `repair_chain`, if `fs::write(&tmp_path, &new_body)` succeeds but `sync_all` on the temp file fails, the temp file at `events.jsonl.tmp` is left on disk with no cleanup.
- **Good behavior:** Given a sync failure during repair, the temp file is removed before returning the error.
- **Forbidden behavior:** The error path must not leave the events file in an inconsistent state (H-001 covers this) and must not leak files.
- **Files involved:** `rust/task-registry-flow-cli/src/verify_chain.rs`
- **Positive test:** Successful repair → no `.tmp` file left behind
- **Negative test:** N/A — this is a defensive cleanup on an OS-level error path; structurally confirmed by code review
- **Domain/API/UI:** N/A — internal hygiene
- **Runtime:** `verify-chain --repair` (success) → `events.jsonl.tmp` does not exist after command completes

## Validation Plan

**Focused:**
- `.codex/scripts/task-registry verify-chain --repair --repair` → exit 1 (H-002)
- `.codex/scripts/task-registry verify-chain --format garbage 2>&1 | grep -v 'Some('` (H-003)
- `.codex/scripts/task-registry verify-chain --repair` + inspect output for repair lines (H-004)
- `test -f docs/task-registry/events.jsonl.tmp && echo "FAIL: orphaned tmp" || echo "PASS"` (H-005)
- `.codex/scripts/task-registry source-limit check`
- `cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --test-threads=4`

**Full:**
- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

All files well under 1600 lines:
- `verify_chain.rs`: ~275 lines → estimated +30 lines for all changes → ~305 (limit: 1600)

## Walkthrough Evidence

1. `verify-chain --repair --repair` → exit 1, "duplicate flag: --repair" (H-002 fixed)
2. `verify-chain --format garbage` → exit 1, error contains `"garbage"` not `Some("garbage")` (H-003 fixed)
3. `verify-chain --repair` (tampered chain) → text output includes repair count and per-line details (H-004 fixed)
4. No `events.jsonl.tmp` orphaned after repair (H-005 fixed)
5. `repair_chain` uses `truncate(false)` — original file untouched until rename (H-001 fixed)

## Task Manifest

```toml
schema_version = 2
plan_id = "verify-chain-hardening-batch-2-2026-05-31"

[[behaviors]]
behavior_id = "B-H1-repair-does-not-truncate-before-lock"
gap_id = "H-001"
polarity = "positive"
title = "repair_chain does not truncate events file before safe write"
given = "a tampered events file"
when = "verify-chain --repair is run"
then = "the events file is repaired via atomic rename; the original file is never truncated to zero before the rename succeeds"
confirmation = ".codex/scripts/task-registry verify-chain --repair 2>&1; test $? -eq 0"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --repair 2>&1; test $? -eq 0"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H1-repair-preserves-on-lock-failure-negative"
gap_id = "H-001"
polarity = "negative"
title = "repair_chain does not truncate on lock contention"
given = "a tampered events file locked by another process"
when = "verify-chain --repair is run while the file is locked"
then = "the original events file content is preserved; the error is reported; the file is not truncated"
confirmation = "code-review: truncate(true) changed to truncate(false) on the OpenOptions builder"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- verify_chain -- --test-threads=4"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H2-rejects-duplicate-repair-flag"
gap_id = "H-002"
polarity = "positive"
title = "parse_verify_chain_args rejects duplicate --repair"
given = "the verify-chain subcommand"
when = "--repair is passed twice"
then = "the CLI exits 1 with an error mentioning duplicate flag"
confirmation = ".codex/scripts/task-registry verify-chain --repair --repair 2>&1; test $? -eq 1"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --repair --repair 2>&1; test $? -eq 1"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H2-rejects-duplicate-format-flag"
gap_id = "H-002"
polarity = "positive"
title = "parse_verify_chain_args rejects duplicate --format"
given = "the verify-chain subcommand"
when = "--format is passed twice"
then = "the CLI exits 1 with an error mentioning duplicate flag"
confirmation = ".codex/scripts/task-registry verify-chain --format json --format json 2>&1; test $? -eq 1"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --format json --format json 2>&1; test $? -eq 1"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H2-single-flags-still-accepted-negative"
gap_id = "H-002"
polarity = "negative"
title = "single occurrences of --repair and --format still work"
given = "the verify-chain subcommand"
when = "--repair or --format json is passed exactly once"
then = "the CLI processes normally"
confirmation = ".codex/scripts/task-registry verify-chain --repair --format json 2>&1; test $? -eq 0"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --repair --format json 2>&1; test $? -eq 0"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H3-no-option-debug-in-errors"
gap_id = "H-003"
polarity = "positive"
title = "format value errors do not leak Option debug formatting"
given = "the verify-chain subcommand"
when = "--format is passed an invalid value or no value"
then = "the error message uses user-friendly phrasing without Some() or None"
confirmation = ".codex/scripts/task-registry verify-chain --format garbage 2>&1 | grep -qv 'Some('"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --format garbage 2>&1; test $? -eq 1"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H3-valid-format-still-accepted-negative"
gap_id = "H-003"
polarity = "negative"
title = "valid --format json still works after error message cleanup"
given = "the verify-chain subcommand"
when = "--format json is passed"
then = "the CLI returns JSON output"
confirmation = ".codex/scripts/task-registry verify-chain --format json 2>&1; test $? -eq 0"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --format json 2>&1; test $? -eq 0"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H4-repair-output-shows-pass-diagnostics"
gap_id = "H-004"
polarity = "positive"
title = "repair text output surfaces what was fixed"
given = "a tampered events file"
when = "verify-chain --repair runs and fixes lines"
then = "text output includes repair count and per-line details for each repaired line"
confirmation = "code-review: format_chain_report iterates CheckStatus::Pass diagnostics when summary.pass > 0"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- verify_chain_repair -- --test-threads=4"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H4-no-spurious-pass-on-intact-chain-negative"
gap_id = "H-004"
polarity = "negative"
title = "intact chain does not show spurious repair lines"
given = "an intact events file with no breaks"
when = "verify-chain runs in text mode"
then = "output shows 'receipt chain is intact' without repair lines"
confirmation = ".codex/scripts/task-registry verify-chain 2>&1 | head -1"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain 2>&1; test $? -eq 0"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H5-no-orphaned-tmp-after-repair"
gap_id = "H-005"
polarity = "positive"
title = "successful repair leaves no orphaned temp file"
given = "a tampered events file"
when = "verify-chain --repair completes successfully"
then = "no events.jsonl.tmp file exists alongside events.jsonl"
confirmation = "test ! -f docs/task-registry/events.jsonl.tmp"

[[behaviors.verifiers]]
type = "command"
command = "test ! -f docs/task-registry/events.jsonl.tmp"
expected_exit = 0

[[behaviors]]
behavior_id = "B-H5-tmp-file-cleaned-on-error-path-negative"
gap_id = "H-005"
polarity = "negative"
title = "temp file is cleaned up on write/sync failure"
given = "a repair attempt where sync_all fails"
when = "the error path executes"
then = "the tmp file is removed before returning the error"
confirmation = "code-review: fs::remove_file(&tmp_path) is called on the sync_all error path"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --manifest-path rust/task-registry-flow-cli/Cargo.toml -- verify_chain -- --test-threads=4"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-VH2-001"
behavior_ids = ["B-H1-repair-does-not-truncate-before-lock", "B-H1-repair-preserves-on-lock-failure-negative"]
title = "Fix repair_chain truncation hazard — use truncate(false)"
reason = "repair_chain opens the events file with truncate(true) before acquiring the lock, creating a data-loss hazard if any step before the atomic rename fails"
acceptance_proof = "Behavior B-H1: repair_chain OpenOptions uses truncate(false); events file survives lock-contention error paths intact"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "repair_chain OpenOptions"
required_change = "change truncate(true) to truncate(false) so the events file is not zeroed before the atomic rename"

[[tasks]]
task_id = "TASK-2026-05-31-VH2-002"
behavior_ids = ["B-H2-rejects-duplicate-repair-flag", "B-H2-rejects-duplicate-format-flag", "B-H2-single-flags-still-accepted-negative", "B-H3-no-option-debug-in-errors", "B-H3-valid-format-still-accepted-negative"]
title = "Reject duplicate flags and fix Option debug in error messages"
reason = "parse_verify_chain_args silently accepts duplicate --repair/--format and leaks Rust Option debug formatting in error messages"
acceptance_proof = "Behavior B-H2: --repair --repair exits 1 with duplicate flag error; Behavior B-H3: error messages use user-friendly phrasing without Some()/None"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "parse_verify_chain_args"
required_change = "detect duplicate --repair and --format flags; format error messages without Option debug representation"

[[tasks]]
task_id = "TASK-2026-05-31-VH2-003"
behavior_ids = ["B-H4-repair-output-shows-pass-diagnostics", "B-H4-no-spurious-pass-on-intact-chain-negative"]
title = "Render pass diagnostics in format_chain_report text output"
reason = "After --repair, format_chain_report ignores pass diagnostics, so users never see what lines were fixed"
acceptance_proof = "Behavior B-H4: repair output includes repair count and per-line details"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "format_chain_report"
required_change = "render pass diagnostics when summary.pass > 0, showing repair count and per-line details"

[[tasks]]
task_id = "TASK-2026-05-31-VH2-004"
behavior_ids = ["B-H5-no-orphaned-tmp-after-repair", "B-H5-tmp-file-cleaned-on-error-path-negative"]
title = "Clean up orphaned temp file on sync failure in repair_chain"
reason = "If sync_all fails after a successful fs::write of the temp file, the events.jsonl.tmp file is left on disk"
acceptance_proof = "Behavior B-H5: successful repair leaves no .tmp file; error path cleans up"
status = "planned"
kind = "implementation"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "repair_chain error handling"
required_change = "add fs::remove_file(&tmp_path) cleanup on the sync_all error path"
```
