# Secondary Governance Hardening Gap Closure Contract

## Approved Scope

Close two additional hardening gaps in this repo only:

1. Mutation hook command parsing does not treat compact shell redirections such as `>src/lib.rs` or variable compact redirections such as `>$target` as write intent.
2. Release-source required files can be accepted through filesystem symlinks instead of native release source files.

Out of scope: new public plugin features, legacy compatibility behavior, broad parser rewrites, and changes outside `/home/hasnamuss/reclaimed/work/Governance-plugin`.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/secondary-hardening-2026-05-31.md` - `closure_contract`: activate this contract before implementation; acceptance proof: `.codex/scripts/task-registry activate docs/plans/secondary-hardening-2026-05-31.md`.
- [ ] `[VERIFY]` `.codex/agent-governance.toml` - `workspace_boundary`: confirm repo root matches the configured mutation root; acceptance proof: `pwd && git rev-parse --show-toplevel`.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli/src/tests/hook_command_tests.rs` - `line_budget`: extend focused test modules without growing `tests/mod.rs`; acceptance proof: `.codex/scripts/task-registry source-limit check`.

### Phase 1: Hook command parser hardening
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/mutation_hook.rs` - `compact_redirection_detection`: extract deterministic compact redirect paths and deny compact variable redirects.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/mutation_hook.rs` - `inline_open_write_detection`: scan every inline `open(...)` call and fail closed when a write-mode target is not deterministic.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/hook_command_tests.rs` - `compact_redirection_suite`: add positive and negative compact redirect tests.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/hook_command_tests.rs` - `mixed_inline_open_suite`: prove read-only extracted paths cannot mask a variable write target.

### Phase 2: Release-source symlink hardening
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `required_file_native_source`: reject symlinked required release files.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/release_source_tests.rs` - `required_symlink_negative`: add a negative fixture for symlinked release-source artifacts.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `release_native_file_contract`: document that release-source required artifacts must be native files, not symlinks.

### Phase 3: Validation and handoff
- [ ] `[VERIFY]` `rust/task-registry-flow-cli` - `focused_behavior`: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_ release_source_rejects_required_symlink` cannot run as one cargo filter, so run the two focused commands listed in the validation plan.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli` - `full_rust`: `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`.
- [ ] `[VERIFY]` `scripts/status.sh` - `posture`: `scripts/status.sh --strict`.

## Per-Gap Success Criteria

### GAP-SH01: Compact and mixed inline command writes fail closed
- Current failure: A shell command can write via compact redirect syntax, or hide a variable write behind a deterministic read path, without triggering deterministic target enforcement.
- Good behavior: Given compact redirection to an active task target, when the hook runs, then the command is allowed because the target is extracted and authorized.
- Forbidden behavior: Given compact redirection to an unbound path, compact variable redirection, or mixed inline read/write where the write target is variable, when the hook runs, then it fails with the deterministic target error.
- Files involved: `rust/task-registry-flow-cli/src/mutation_hook.rs`, `rust/task-registry-flow-cli/src/tests/hook_command_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_compact_redirection_with_active_target`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_compact_` and `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_mixed_open_`.
- Data/schema/provenance: No schema change; hook denial receipts remain schema version 2.
- Runtime: Hook command inspection must fail closed before registry target matching when a write target is not deterministic.

### GAP-SH02: Release-source required artifacts must be native files
- Current failure: A required release-source path can be a symlink and still satisfy `release-file-present`.
- Good behavior: Given a native required release file, when `release-check all` runs, then it passes `release-file-present`.
- Forbidden behavior: Given a symlink at a required release-source path, when `release-check required` runs, then it fails `release-file-present` with actual `symlink`.
- Files involved: `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/tests/release_source_tests.rs`, `docs/runtime-schemas.md`.
- Positive test: `.codex/scripts/task-registry release-check all --format json`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_required_symlink`.
- Data/schema/provenance: Existing check id remains canonical; no compatibility alias or new schema version.
- Runtime: Release gates must inspect symlink metadata rather than following links.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_compact_redirection_with_active_target`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_compact_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_mixed_open_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_required_symlink`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry release-check all --format json`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `scripts/status.sh --strict`

## Walkthrough Evidence

Capture after implementation:
- `TASK_REPORT PLAN-2026-05-31-secondary-hardening`: all tasks completed, no deferred or blocked tasks.
- `TASK_METRICS`: `active=0`, `deferred=0`, `blocked=0`, `receipt_chain_breaks=0`, `unchained_events=0`.
- Focused negative tests reject compact unbound redirects, compact variable redirects, mixed inline variable writes, and symlinked release-source files.
- Full validation commands exit zero.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-secondary-hardening"

[[behaviors]]
behavior_id = "B-SH01-positive"
gap_id = "GAP-SH01"
polarity = "positive"
title = "Compact redirection to an active target is authorized"
given = "An active registry task target and a shell command using compact redirection to that exact file"
when = "verify-mutation-hook evaluates the command payload"
then = "The hook extracts the target and allows the payload"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_compact_redirection_with_active_target"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_allows_compact_redirection_with_active_target"
expected_exit = 0

[[behaviors]]
behavior_id = "B-SH01-negative-compact"
gap_id = "GAP-SH01"
polarity = "negative"
title = "Compact redirection without authorization is denied"
given = "A shell command using compact redirection to an unbound path or variable target"
when = "verify-mutation-hook evaluates the command payload"
then = "The hook denies the payload"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_compact_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_compact_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-SH01-negative-open"
gap_id = "GAP-SH01"
polarity = "negative"
title = "Mixed inline open writes with variable targets are denied"
given = "An inline Python command that reads an active target but writes to a variable target"
when = "verify-mutation-hook evaluates the command payload"
then = "The hook denies the payload instead of trusting the read path"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_mixed_open_"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_command_denies_mixed_open_"
expected_exit = 0

[[behaviors]]
behavior_id = "B-SH02-positive"
gap_id = "GAP-SH02"
polarity = "positive"
title = "Native release-source files pass release checks"
given = "The current release-source manifest and native required files"
when = "release-check all runs"
then = "The release-source report has zero failures"
confirmation = ".codex/scripts/task-registry release-check all --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check all --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-SH02-negative"
gap_id = "GAP-SH02"
polarity = "negative"
title = "Symlinked release-source files are rejected"
given = "A release-source fixture with a symlink at a required file path"
when = "release-check required runs"
then = "The release-file-present diagnostic fails with actual symlink"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_required_symlink"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_source_rejects_required_symlink"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-SH01"
status = "planned"
kind = "authorization"
reason = "Mutation hooks must fail closed for compact shell redirections and hidden inline write targets."
behavior_ids = ["B-SH01-positive", "B-SH01-negative-compact", "B-SH01-negative-open"]
title = "Harden command write-target extraction"
acceptance_proof = "Behaviors B-SH01-positive, B-SH01-negative-compact, and B-SH01-negative-open pass."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "compact_redirection_detection"
required_change = "Extract compact redirect targets and mark variable compact redirects as ambiguous writes."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "inline_open_write_detection"
required_change = "Scan every inline open call and reject write-mode calls whose target is not deterministic."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/hook_command_tests.rs"
object = "command_write_negative_suite"
required_change = "Add compact redirect and mixed inline open tests."

[[tasks]]
task_id = "TASK-2026-05-31-SH02"
status = "planned"
kind = "release"
reason = "Release-source required files must be native files, not symlink indirections."
behavior_ids = ["B-SH02-positive", "B-SH02-negative"]
title = "Reject symlinked release-source required files"
acceptance_proof = "Behaviors B-SH02-positive and B-SH02-negative pass."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "required_file_native_source"
required_change = "Use symlink metadata and fail required release-source paths that are symlinks."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/release_source_tests.rs"
object = "required_symlink_negative"
required_change = "Add a symlinked required-file negative fixture."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "release_native_file_contract"
required_change = "Document that release-source required artifacts must be native files."
```
