# Codebase Gap Closure Contract

## Approved Scope

**Batch 1 (active)** — 5 high-impact, clearly-fixable gaps:
1. No `fsync` after atomic writes (crash-safety)
2. `.unwrap()` in CLI output path (potential panic)
3. JSON injection risk in hook response (security)
4. `CLAUDE.md` line count inaccuracy (documentation trust)
5. `normalize_relative_path` duplicated across modules (code quality)

**Batches 2–6 (deferred)** — 21 remaining gaps listed at end of this contract. Each will be addressed in its own batch of 5 (final batch: 1).

## Phased Required Change Checklist

### Batch 1: Crash-safety, correctness, and hygiene

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/registry_io.rs` — `save_registry`: add `sync_all()` after temp file write before rename
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/verify_chain.rs` — `repair_chain`: add `sync_all()` after temp file write before rename
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` — `append_event`: add `sync_all()` after writeln before file drop
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cli.rs` — replace 3 `.unwrap()` calls with fallback error handling
- [ ] `[MODIFY]` `tools/agent-governance/pre-tool-use-gap-closure.sh` — `emit_json`: replace fragile sed escaping with proper JSON construction
- [ ] `[MODIFY]` `CLAUDE.md` — fix `render-from-config.sh` line count from "~24k lines" to "655 lines"
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/source_limit.rs` — remove duplicate `normalize_relative_path`, import from `runtime`
- [ ] `[VERIFY]` `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- [ ] `[VERIFY]` `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- [ ] `[VERIFY]` `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- [ ] `[VERIFY]` `bash -n tools/agent-governance/pre-tool-use-gap-closure.sh`
- [ ] `[VERIFY]` `bash -n scripts/pre-tool-use-gap-closure.sh`
- [ ] `[VERIFY]` `cargo run --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- source-limit check`

## Per-Gap Success Criteria

### Gap 1: No `fsync` after atomic writes
- **Current failure**: Temp file written then renamed without `sync_all()`. OS crash can leave zero-length or partial file at canonical path.
- **Good behavior**: After writing temp file, `sync_all()` is called before `rename`. Data is durable on disk before the atomic swap.
- **Forbidden behavior**: Partial or empty `docs/task-registry.toml` or `docs/task-registry/events.jsonl` after crash.
- **Files involved**: `registry_io.rs`, `verify_chain.rs`, `runtime.rs`
- **Positive test**: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- **Negative test**: N/A — crash testing is not practical in unit tests
- **Runtime**: N/A — crash-safety is not runtime-observable without OS-level testing

### Gap 2: `.unwrap()` in CLI output path
- **Current failure**: Three `.unwrap()` calls on `serde_json::to_string()` in `cli.rs`. If serialization fails, the process panics.
- **Good behavior**: Serialization failure produces a fallback JSON error message to stdout, process exits with code 1.
- **Forbidden behavior**: Process panic from serialization failure in the output path.
- **Files involved**: `cli.rs`
- **Positive test**: Existing CLI JSON envelope tests pass. Manual inspection of fallback path.
- **Negative test**: N/A — serialization failure of `CommandReport` requires internal struct corruption

### Gap 3: JSON injection in hook response
- **Current failure**: `emit_json` in the mutation gate uses `sed 's/\\/\\\\/g; s/"/\\"/g'` for JSON escaping. Control characters (tabs, carriage returns, newlines beyond the first) produce malformed JSON.
- **Good behavior**: All JSON string values are properly escaped per RFC 8259. Error messages with any characters produce valid JSON.
- **Forbidden behavior**: Malformed JSON emitted by the hook gate.
- **Files involved**: `tools/agent-governance/pre-tool-use-gap-closure.sh`
- **Positive test**: `bash -n tools/agent-governance/pre-tool-use-gap-closure.sh` passes syntax check
- **Negative test**: Feed a reason string with tabs and newlines; verify output is valid JSON

### Gap 4: CLAUDE.md line count inaccuracy
- **Current failure**: CLAUDE.md says `render-from-config.sh` is "~24k lines". It is 655 lines.
- **Good behavior**: CLAUDE.md accurately describes `render-from-config.sh` as "655 lines".
- **Forbidden behavior**: Inaccurate documentation that misleads capacity planning.
- **Files involved**: `CLAUDE.md`
- **Positive test**: `wc -l scripts/render-from-config.sh` returns 655
- **Negative test**: N/A

### Gap 5: `normalize_relative_path` duplicated
- **Current failure**: Identical logic exists in `source_limit.rs:592` and `runtime.rs:396`. The `source_limit.rs` version lacks the glob metacharacter check present in `runtime.rs`.
- **Good behavior**: Single implementation in `runtime.rs`, imported by `source_limit.rs`. Glob metacharacter check applies uniformly.
- **Forbidden behavior**: Divergent behavior between the two copies.
- **Files involved**: `source_limit.rs`, `runtime.rs`
- **Positive test**: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- **Negative test**: N/A

## Validation Plan

Focused:
```bash
cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml
cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check
cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings
bash -n tools/agent-governance/pre-tool-use-gap-closure.sh
bash -n scripts/pre-tool-use-gap-closure.sh
cargo run --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- source-limit check
```

Full:
```bash
bash scripts/test-install-modes.sh
bash scripts/test-release-readiness.sh
```

## Walkthrough Evidence
- All tests pass
- Source limit check passes
- Shell syntax checks pass
- Clippy with deny warnings passes

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-gap-closure-batch-1"

[[behaviors]]
behavior_id = "B-001-fsync-atomic-writes"
gap_id = "GAP-001"
polarity = "positive"
title = "Atomic writes are crash-safe via fsync"
given = "a registry or events file being written"
when = "save_registry, repair_chain, or append_event writes data"
then = "sync_all() is called before rename or file close"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-001b-no-stale-tmp"
gap_id = "GAP-001"
polarity = "negative"
title = "No stale .tmp file lingers after registry or events save"
given = "a registry or events file save completes"
when = "save_registry, repair_chain, or append_event finishes"
then = "no .tmp file remains at the canonical path"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-no-unwrap-cli"
gap_id = "GAP-002"
polarity = "positive"
title = "CLI output path handles serialization failure without panic"
given = "a CommandReport that fails to serialize"
when = "the CLI writes JSON output"
then = "a fallback error message is produced instead of a panic"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002b-no-unwrap-in-output-path"
gap_id = "GAP-002"
polarity = "negative"
title = "CLI output path contains no .unwrap() calls"
given = "cli.rs source code"
when = "reviewing the output rendering functions"
then = "no .unwrap() call exists in main_entry, render_error, or failure_json_for_test"
confirmation = "! grep -cF '.unwrap()' rust/task-registry-flow-cli/src/cli.rs"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test $(grep -cF \".unwrap()\" rust/task-registry-flow-cli/src/cli.rs) -eq 0'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-json-safe-hook"
gap_id = "GAP-003"
polarity = "positive"
title = "Hook gate emits valid JSON for any reason string"
given = "an error reason containing control characters, tabs, or special chars"
when = "the mutation hook gate emits a deny response"
then = "the output is valid JSON"
confirmation = "bash -n tools/agent-governance/pre-tool-use-gap-closure.sh"

[[behaviors.verifiers]]
type = "contains"
path = "tools/agent-governance/pre-tool-use-gap-closure.sh"
needle = "python3"

[[behaviors]]
behavior_id = "B-003b-hook-produces-valid-json"
gap_id = "GAP-003"
polarity = "negative"
title = "Hook gate produces valid JSON even with control characters in reason"
given = "a reason string containing tabs, carriage returns, and backslashes"
when = "emit_json deny is called"
then = "the output is parseable JSON"
confirmation = "bash -n tools/agent-governance/pre-tool-use-gap-closure.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -n tools/agent-governance/pre-tool-use-gap-closure.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-claude-md-accurate"
gap_id = "GAP-004"
polarity = "positive"
title = "CLAUDE.md accurately describes render-from-config.sh line count"
given = "CLAUDE.md documentation"
when = "a reader checks the line count of render-from-config.sh"
then = "it matches the actual file (655 lines)"
confirmation = "wc -l scripts/render-from-config.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'lines=$(wc -l < scripts/render-from-config.sh); test \"$lines\" -eq 655'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004b-no-misleading-line-count"
gap_id = "GAP-004"
polarity = "negative"
title = "CLAUDE.md does not claim render-from-config.sh is ~24k lines"
given = "CLAUDE.md documentation"
when = "a reader checks render-from-config.sh description"
then = "it does not state ~24k lines"
confirmation = "! grep -c '~24k lines' CLAUDE.md"

[[behaviors.verifiers]]
type = "command"
command = "bash -c 'test $(grep -c \"~24k lines\" CLAUDE.md) -eq 0'"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-dedup-normalize-path"
gap_id = "GAP-005"
polarity = "positive"
title = "normalize_relative_path has a single implementation"
given = "source_limit.rs needing path normalization"
when = "normalize_relative_path is called"
then = "it uses the runtime.rs implementation with glob metacharacter check"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005b-no-duplicate-def"
gap_id = "GAP-005"
polarity = "negative"
title = "source_limit.rs does not contain a duplicate normalize_relative_path definition"
given = "source_limit.rs source code"
when = "the module is compiled"
then = "no duplicate fn normalize_relative_path exists in source_limit.rs"
confirmation = "! grep -q 'fn normalize_relative_path' rust/task-registry-flow-cli/src/source_limit.rs"

[[behaviors.verifiers]]
type = "command"
command = "bash -c '! grep -q \"fn normalize_relative_path\" rust/task-registry-flow-cli/src/source_limit.rs'"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-001"
status = "planned"
kind = "implementation"
reason = "Temp file writes without fsync risk zero-length or partial registry/events after OS crash"
behavior_ids = ["B-001-fsync-atomic-writes"]
title = "Add fsync to atomic writes in registry_io, verify_chain, runtime"
acceptance_proof = "Behavior B-001: cargo test passes, sync_all() calls present after temp file writes"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/registry_io.rs"
object = "save_registry"
required_change = "Add sync_all() after temp file write before rename"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verify_chain.rs"
object = "repair_chain"
required_change = "Add sync_all() after temp file write before rename"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "append_event"
required_change = "Add sync_all() after writeln before file drop"

[[tasks]]
task_id = "TASK-2026-05-31-002"
status = "planned"
kind = "implementation"
reason = ".unwrap() in CLI output path could panic on serialization failure"
behavior_ids = ["B-002-no-unwrap-cli"]
title = "Replace .unwrap() in CLI output with fallback error handling"
acceptance_proof = "Behavior B-002: zero .unwrap() calls in cli.rs output path, tests pass"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "main_entry and render_error"
required_change = "Replace 3 .unwrap() calls with .unwrap_or_else() fallback error handling"

[[tasks]]
task_id = "TASK-2026-05-31-003"
status = "planned"
kind = "implementation"
reason = "Fragile sed escaping in hook gate can produce malformed JSON with control characters"
behavior_ids = ["B-003-json-safe-hook"]
title = "Fix JSON injection in hook gate emit_json function"
acceptance_proof = "Behavior B-003: shell syntax check passes, JSON escaping handles control characters"

[[tasks.targets]]
file = "tools/agent-governance/pre-tool-use-gap-closure.sh"
object = "emit_json"
required_change = "Replace fragile sed escaping with python3 json.dumps (+ tab/CR sed fallback)"

[[tasks]]
task_id = "TASK-2026-05-31-004"
status = "planned"
kind = "documentation"
reason = "CLAUDE.md overstates render-from-config.sh at ~24k lines; actual is 655, eroding doc trust"
behavior_ids = ["B-004-claude-md-accurate"]
title = "Fix CLAUDE.md line count for render-from-config.sh"
acceptance_proof = "Behavior B-004: CLAUDE.md says 655 lines, matches wc -l"

[[tasks.targets]]
file = "CLAUDE.md"
object = "render-from-config.sh description"
required_change = "Change ~24k lines to 655 lines"

[[tasks]]
task_id = "TASK-2026-05-31-005"
status = "planned"
kind = "implementation"
reason = "Identical normalize_relative_path logic in source_limit.rs and runtime.rs wastes the 1600-line budget"
behavior_ids = ["B-005-dedup-normalize-path", "B-005b-no-duplicate-def"]
title = "Deduplicate normalize_relative_path from source_limit.rs"
acceptance_proof = "Behavior B-005: single definition in runtime.rs, tests pass, no duplicate in source_limit.rs"

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/source_limit.rs"
object = "normalize_relative_path"
required_change = "Remove duplicate fn, import from crate::runtime instead"
```

---

## Deferred Gaps (Batches 2–6)

### Batch 2 candidates
6. Verify chain `_broken` parameter unused — use it for targeted repair
7. `bash -lc` in verifier and hook gate — switch to `bash -c`
8. `VERIFY_CHAIN` opcode missing from skill definitions
9. `{{REPO_SLUG}}` used in template but never defined
10. Temp file collision risk in `status.sh`

### Batch 3 candidates
11. `plan_requires_decomposition` / `task_requires_decomposition` duplicated logic
12. `archive_completed` misleading function name
13. `Derive_plan_status` doesn't compute Planned/Blocked plan-level status
14. Version "2.0.0" hardcoded in 4+ shell scripts
15. No read lock during registry load (TOCTOU)

### Batch 4 candidates
16. `rust_bucket` classifier uses fragile prefix matching
17. Platform hook coverage asymmetry (Codex vs Antigravity)
18. Dead code masked by `#[allow(dead_code)]` in `string_enum!` macro
19. No doc comments on public API
20. `tests/mod.rs` at 1,547 lines (53 from limit)

### Batch 5 candidates
21. 7 test files referenced in archives but absent on disk
22–25. Test coverage gaps (defer_task, verify-mutation-hook, verify-chain edge cases, Cursor format hooks, concurrent writes)

### Batch 6 candidates
26. CI: no Rust build artifact caching, no fuzzing, no matrix strategy
