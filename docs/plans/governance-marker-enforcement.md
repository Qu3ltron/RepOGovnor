# Governance Marker Enforcement Gap Closure Contract

## Approved Scope

Close the dry-run/status gap where `AGENTS.md` or `GEMINI.md` without an
`agent-governance` marker block can still be reported as acceptable posture.

In scope:
- Treat missing governance marker blocks as failed posture, not notes.
- Make canonical full-file projections install exactly one marker block.
- Extend typed runtime status diagnostics so JSON status checks also fail when
  governance markers are missing or malformed.
- Add negative migration coverage for markerless workspaces.

Out of scope:
- Changing hook authorization semantics beyond marker diagnostics.
- Adding backward-compatibility adapters for markerless workspaces.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/governance-marker-enforcement.md` - `closure_contract`: define approved scope, success criteria, validation, and manifest; acceptance proof: `PLAN_ACTIVATE docs/plans/governance-marker-enforcement.md ok`.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `active_plan`: activate this manifest before implementation edits; acceptance proof: `.codex/scripts/task-registry validate`.

### Phase 1: Runtime and status enforcement
- [ ] `[MODIFY]` `scripts/status.sh` - `check_markers`: report missing marker pairs as `FAIL`, not `NOTE`; acceptance proof: behavior `B-2026-05-31-marker-status-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/status_checks.rs` - `marker_check`: include typed marker diagnostics for `AGENTS.md` and `GEMINI.md`; acceptance proof: behaviors `B-2026-05-31-marker-status-positive` and `B-2026-05-31-marker-status-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `status_marker_tests`: assert marker-present success and missing/malformed marker failures; acceptance proof: behavior `B-2026-05-31-marker-status-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/status_check_tests.rs` - `status_marker_tests`: keep status-check tests below the source file limit; acceptance proof: `.codex/scripts/task-registry source-limit check`.

### Phase 2: Canonical projection
- [ ] `[MODIFY]` `templates/AGENTS.md.template` - `marker_block`: install one governed marker block in full-file projections; acceptance proof: behavior `B-2026-05-31-marker-template-positive`.
- [ ] `[MODIFY]` `templates/GEMINI.md.template` - `marker_block`: install one governed marker block in full-file projections; acceptance proof: behavior `B-2026-05-31-marker-template-positive`.
- [ ] `[MODIFY]` `AGENTS.md` - `self_projection`: align this repo with the marker-bearing canonical projection; acceptance proof: `scripts/status.sh --strict`.
- [ ] `[MODIFY]` `GEMINI.md` - `self_projection`: align this repo with the marker-bearing canonical projection; acceptance proof: `scripts/status.sh --strict`.

### Phase 3: Negative migration and release gates
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `markerless_workspace_negative`: prove markerless workspaces fail strict posture and dry-run projects marker installation without mutating files; acceptance proof: behavior `B-2026-05-31-marker-template-negative`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `marker_release_checks`: assert release templates and self-projected docs carry marker blocks; acceptance proof: behavior `B-2026-05-31-marker-validation`.
- [ ] `[MODIFY]` `README.md` - `status_contract`: document that markerless `AGENTS.md` or `GEMINI.md` is unaligned posture; acceptance proof: behavior `B-2026-05-31-marker-validation`.

## Per-Gap Success Criteria

### GAP-001: Markerless governance docs pass posture
- Current failure: `scripts/status.sh --strict` reports missing marker blocks as notes and exits successfully.
- Good behavior: Given `AGENTS.md` or `GEMINI.md` without a single marker pair, when status runs, then posture fails with a marker-block diagnostic.
- Forbidden behavior: Markerless docs are never reported as aligned or optional.
- Files involved: `scripts/status.sh`, `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_missing_marker_failure_exits_nonzero`.
- Domain/API/UI: CLI diagnostics remain schema-backed JSON for runtime checks and plain status lines for shell checks.
- Runtime: `status-check --format json` exits nonzero on missing or malformed marker diagnostics.

### GAP-002: Full projections lack marker blocks
- Current failure: force-installed full `AGENTS.md` and `GEMINI.md` can contain governance text without marker blocks, so later posture cannot prove plugin ownership.
- Good behavior: Given a force projection, when the installer renders full docs, then each generated doc contains exactly one marker block.
- Forbidden behavior: Full-file projection cannot be markerless or produce multiple marker pairs.
- Files involved: `templates/AGENTS.md.template`, `templates/GEMINI.md.template`, `AGENTS.md`, `GEMINI.md`, `scripts/test-install-modes.sh`.
- Positive test: `scripts/install-to-workspace.sh --target . --dry-run` reports aligned after this repo is updated.
- Negative test: `bash scripts/test-install-modes.sh` proves markerless temp workspaces fail strict posture before merge/force repair.
- Domain/API/UI: Installed docs expose visible governance instructions plus marker block provenance.
- Runtime: Installer dry-run is non-mutating and reports updates for markerless docs.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_`
- `bash scripts/test-install-modes.sh`
- `scripts/status.sh --strict`
- `.codex/scripts/task-registry source-limit check`

Full:
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-governance-marker-enforcement`

## Source File Limit

Expected line-budget impact stays below the 1600-line hard limit by splitting
status-check tests into `rust/task-registry-flow-cli/src/tests/status_check_tests.rs`.
Run `.codex/scripts/task-registry source-limit check` before completion.

## Walkthrough Evidence

- `scripts/status.sh --strict` fails on a markerless temp workspace in the migration test.
- Self dry-run reports `AGENTS.md` and `GEMINI.md` aligned after marker-bearing projection.
- Runtime JSON status succeeds in this repo and fails in markerless fixtures.
- `TASK_REPORT` and `TASK_METRICS` show no deferred or blocked work.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-governance-marker-enforcement"

[[behaviors]]
behavior_id = "B-2026-05-31-marker-status-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Marker-bearing status succeeds"
given = "A workspace with native skill projection and single AGENTS/GEMINI governance marker blocks"
when = "status-check --format json runs"
then = "The status report has no marker failures and exits zero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-marker-status-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Markerless status fails closed"
given = "A workspace with missing or malformed governance marker blocks"
when = "status-check --format json or strict shell status runs"
then = "The status report fails with marker diagnostics and exits nonzero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_missing_marker_failure_exits_nonzero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_missing_marker_failure_exits_nonzero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-marker-template-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Canonical projections include marker blocks"
given = "The plugin full-file templates are rendered"
when = "install dry-run evaluates this repo"
then = "AGENTS.md and GEMINI.md are aligned and each carries a marker block"
confirmation = "scripts/install-to-workspace.sh --target . --dry-run | tee /tmp/marker-dry-run.out && grep -q 'AGENTS.md: aligned' /tmp/marker-dry-run.out && grep -q 'GEMINI.md: aligned' /tmp/marker-dry-run.out"

[[behaviors.verifiers]]
type = "command"
command = "scripts/install-to-workspace.sh --target . --dry-run | tee /tmp/marker-dry-run.out && grep -q 'AGENTS.md: aligned' /tmp/marker-dry-run.out && grep -q 'GEMINI.md: aligned' /tmp/marker-dry-run.out"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-marker-template-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Markerless migration is repaired"
given = "A temporary workspace starts with markerless AGENTS.md"
when = "installer migration tests run"
then = "Strict status rejects the markerless state and merge/force projections repair it"
confirmation = "bash scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-marker-validation"
gap_id = "GAP-002"
polarity = "validation"
title = "Full marker enforcement validation passes"
given = "Marker enforcement is implemented"
when = "Release readiness and repo posture gates run"
then = "All marker, release, source-limit, and registry checks pass"
confirmation = "bash scripts/test-release-readiness.sh all && scripts/status.sh --strict && .codex/scripts/task-registry source-limit check"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all && scripts/status.sh --strict && .codex/scripts/task-registry source-limit check"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-marker-enforcement-001"
title = "Fail status on markerless governance docs"
status = "planned"
kind = "diagnostics"
reason = "Markerless AGENTS/GEMINI files currently pass status as notes."
acceptance_proof = "Behaviors B-2026-05-31-marker-status-positive and B-2026-05-31-marker-status-negative pass."
behavior_ids = ["B-2026-05-31-marker-status-positive", "B-2026-05-31-marker-status-negative"]

[[tasks.targets]]
file = "scripts/status.sh"
object = "check_markers"
required_change = "Report missing governance marker blocks as failures."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "marker_check"
required_change = "Add typed marker diagnostics for AGENTS.md and GEMINI.md."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "status_marker_tests"
required_change = "Declare focused status-check test module without exceeding the source limit."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/status_check_tests.rs"
object = "status_marker_tests"
required_change = "Assert marker-bearing success and missing/malformed marker failures."

[[tasks]]
task_id = "TASK-2026-05-31-marker-enforcement-002"
title = "Install marker-bearing full projections"
status = "planned"
kind = "migration"
reason = "Force projections can currently produce governance docs without marker provenance."
acceptance_proof = "Behaviors B-2026-05-31-marker-template-positive and B-2026-05-31-marker-template-negative pass."
behavior_ids = ["B-2026-05-31-marker-template-positive", "B-2026-05-31-marker-template-negative"]

[[tasks.targets]]
file = "templates/AGENTS.md.template"
object = "marker_block"
required_change = "Add exactly one agent-governance marker block to full AGENTS projections."

[[tasks.targets]]
file = "templates/GEMINI.md.template"
object = "marker_block"
required_change = "Add exactly one agent-governance marker block to full GEMINI projections."

[[tasks.targets]]
file = "AGENTS.md"
object = "self_projection"
required_change = "Refresh self projection with marker block."

[[tasks.targets]]
file = "GEMINI.md"
object = "self_projection"
required_change = "Refresh self projection with marker block."

[[tasks]]
task_id = "TASK-2026-05-31-marker-enforcement-003"
title = "Cover marker enforcement in migration and release gates"
status = "planned"
kind = "test"
reason = "Negative migration and release checks must prevent markerless regression."
acceptance_proof = "Behavior B-2026-05-31-marker-validation passes."
behavior_ids = ["B-2026-05-31-marker-validation"]

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "markerless_workspace_negative"
required_change = "Assert strict status fails markerless workspaces and install modes repair markers."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "marker_release_checks"
required_change = "Assert release templates and self docs include governance marker blocks."

[[tasks.targets]]
file = "README.md"
object = "status_contract"
required_change = "Document markerless governance docs as unaligned posture."
```
