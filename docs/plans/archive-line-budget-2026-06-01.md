# Archive Line Budget Gap Closure Contract

## Approved Scope
Close the CI failure where installed plugin smoke checks see generated task-registry archive files over the 1600-line source limit.

In scope:
- Keep generated completed-task archives below the source limit by reducing archive chunk size.
- Add regression coverage proving archive generation stays within the line budget.
- Regenerate task-registry archives through the registry CLI.

Out of scope: changing task history semantics, deleting receipts, or adding release-version changes.

## Phased Required Change Checklist
### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/archive-line-budget-2026-06-01.md` - `Task Manifest`: activate this contract before implementation.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry`: activation, landing, and archive keep validation green.

### Phase 1: Archive budget fix
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - `ARCHIVE_COMPLETED_PLAN_CHUNK_SIZE`: reduce completed-plan archive chunks so generated archive files remain under 1600 lines.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/archive_tests.rs` - `archive budget regression`: prove archive generation keeps each completed archive under `SOURCE_LINE_LIMIT`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `test module list`: include archive budget tests.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: declare the new Rust test source.
- [ ] `[MODIFY]` `docs/task-registry/archive/` - `completed archives`: regenerate completed archives through the registry CLI with smaller chunks.

## Per-Gap Success Criteria
### GAP-001: Installed plugin source-limit failure from generated archives
- Current failure: GitHub CI `Install mode behavior` fails because `plugins/agent-governance/docs/task-registry/archive/completed-001.toml` is 1659 lines after install.
- Good behavior: generated completed archives stay below `SOURCE_LINE_LIMIT`, including when the plugin is cloned under `plugins/agent-governance`.
- Forbidden behavior: archive generation emits any `completed-*.toml` file over 1600 lines.
- Files involved: `rust/task-registry-flow-cli/src/model.rs`, `rust/task-registry-flow-cli/src/tests/archive_tests.rs`, `docs/task-registry/archive/`, `scripts/test-install-modes.sh`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml archive_completed_keeps_archives_under_source_limit -- --nocapture`
- Negative test: `tmp=$(mktemp -d); cp -a . "$tmp/repo"; (cd "$tmp/repo" && git add -A && git -c user.name=Snapshot -c user.email=snapshot@example.invalid commit -qm snapshot && bash scripts/test-install-modes.sh); status=$?; rm -rf "$tmp"; exit $status`
- Domain/API/UI: archive file names may increase in count; registry archive loading remains via `archive_paths`.
- Runtime: `archive-completed`, `validate`, installer smoke, and CI source-limit checks pass.

## Validation Plan
Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml archive_completed_keeps_archives_under_source_limit -- --nocapture`
- `tmp=$(mktemp -d); cp -a . "$tmp/repo"; (cd "$tmp/repo" && git add -A && git -c user.name=Snapshot -c user.email=snapshot@example.invalid commit -qm snapshot && bash scripts/test-install-modes.sh); status=$?; rm -rf "$tmp"; exit $status`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `bash scripts/test-release-readiness.sh all`
- `bash scripts/release-audit.sh`

## Source File Limit
Expected impact is a tiny constant change plus one focused test module. The fix exists specifically to keep generated archive files below 1600 lines in public install smoke checks.

## Walkthrough Evidence
- Contract activation output.
- Archive regression test output.
- Snapshot install-mode smoke output.
- Source-limit, registry validation, release readiness, release audit, and GitHub CI output.

## Task Manifest
```toml
schema_version = 2
plan_id = "PLAN-2026-06-01-archive-line-budget"

[[behaviors]]
behavior_id = "B-001-archive-budget-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Archive generation stays under source limit"
given = "A registry with completed tasks"
when = "archive-completed regenerates completed archive files"
then = "every completed archive file has at most SOURCE_LINE_LIMIT lines"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml archive_completed_keeps_archives_under_source_limit -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml archive_completed_keeps_archives_under_source_limit -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-install-source-limit-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Installed plugin smoke rejects source-limit regressions"
given = "The plugin installed under plugins/agent-governance"
when = "install mode smoke runs strict status checks"
then = "no generated archive violates the source line limit"
confirmation = "tmp=$(mktemp -d); cp -a . \"$tmp/repo\"; (cd \"$tmp/repo\" && git add -A && git -c user.name=Snapshot -c user.email=snapshot@example.invalid commit -qm snapshot && bash scripts/test-install-modes.sh); status=$?; rm -rf \"$tmp\"; exit $status"

[[behaviors.verifiers]]
type = "command"
command = "tmp=$(mktemp -d); cp -a . \"$tmp/repo\"; (cd \"$tmp/repo\" && git add -A && git -c user.name=Snapshot -c user.email=snapshot@example.invalid commit -qm snapshot && bash scripts/test-install-modes.sh); status=$?; rm -rf \"$tmp\"; exit $status"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-01-archive-line-budget-001"
behavior_ids = [
  "B-001-archive-budget-positive",
  "B-002-install-source-limit-negative",
]
status = "planned"
title = "Keep generated registry archives under source limit"
kind = "implementation"
reason = "CI install smoke failed because generated completed archive files exceeded the source line limit."
acceptance_proof = "Behaviors B-001 and B-002."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "archive chunk size"
required_change = "Reduce completed-plan archive chunk size to keep generated archive files under source limit."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/archive_tests.rs"
object = "archive source-limit regression"
required_change = "Assert archive-completed generates archive files at or below SOURCE_LINE_LIMIT."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "test module list"
required_change = "Include archive_tests."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Declare archive_tests as release source."

[[tasks.targets]]
file = "docs/plans/archive-line-budget-2026-06-01.md"
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
required_change = "Regenerate completed archives with all files below the source line limit."
```
