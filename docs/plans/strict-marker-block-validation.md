# Strict Marker Block Validation Gap Closure Contract

## Approved Scope

Close the reviewed marker-enforcement gap where `AGENTS.md` or `GEMINI.md` can pass status by merely mentioning marker tokens or by placing the end marker before the begin marker.

In scope: Rust `status-check`, shell `status.sh`, focused negative tests for prose-only tokens and reversed marker order, registry completion, and commit-ready validation.

Out of scope: changing installer projection content, changing marker names, changing non-status runtime behavior, or adding compatibility shims. The canonical marker contract is exactly one ordered HTML comment block.

Runtime/schema impact: status diagnostics continue to use `governance-marker`; the validation predicate becomes exact and ordered. No new schema version is introduced.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/strict-marker-block-validation.md` - `closure_contract`: declare the exact reviewed gap, success criteria, negative tests, and Task Manifest.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `active_registry`: run `.codex/scripts/task-registry activate docs/plans/strict-marker-block-validation.md`.
- [ ] `[VERIFY]` source budget - `line_budget`: run `wc -l rust/task-registry-flow-cli/src/status_checks.rs rust/task-registry-flow-cli/src/tests/status_check_tests.rs scripts/status.sh scripts/test-install-modes.sh`.

### Phase 1: Runtime enforcement

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/status_checks.rs` - `marker_check`: require exactly one `<!-- agent-governance:begin -->` line before exactly one `<!-- agent-governance:end -->` line.
- [ ] `[MODIFY]` `scripts/status.sh` - `check_markers`: mirror the exact ordered marker block predicate for human and strict status.

### Phase 2: Negative behavior coverage

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/status_check_tests.rs` - `marker_negative_tests`: assert prose-only marker tokens and reversed comment markers fail closed.
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `shell_marker_negative_tests`: assert `status.sh --strict` rejects prose-only tokens and reversed marker order.

### Phase 3: Validation and handoff

- [ ] `[VERIFY]` Rust status tests - run `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_non_block_marker_tokens`.
- [ ] `[VERIFY]` shell install-mode tests - run `scripts/test-install-modes.sh`.
- [ ] `[VERIFY]` full gates - run the validation commands listed below.
- [ ] `[VERIFY]` task registry - complete tasks only after linked verifiers pass, then run `.codex/scripts/task-registry report PLAN-2026-05-31-strict-marker-block-validation` and `.codex/scripts/task-registry metrics`.

## Per-Gap Success Criteria

### GAP-001: Marker validation accepts malformed marker tokens

- Current failure: `status_checks.rs` and `scripts/status.sh` count `agent-governance:begin` and `agent-governance:end` substrings, so prose-only mentions or reversed markers can pass.
- Good behavior: Given `AGENTS.md` and `GEMINI.md` each contain exactly one begin comment line before exactly one end comment line, `status-check --format json` and `status.sh --strict` pass marker checks.
- Forbidden behavior: Given prose-only marker token mentions, reversed end-before-begin markers, duplicate markers, or missing markers, status fails closed with `governance-marker` diagnostics or shell `FAIL` output.
- Files involved: `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/tests/status_check_tests.rs`, `scripts/status.sh`, `scripts/test-install-modes.sh`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_non_block_marker_tokens`.
- Migration test: `scripts/test-install-modes.sh` proves markerless, prose-only, and reversed-marker workspaces fail strict status before repair.
- Data/schema/provenance criteria: no loose prose list becomes authoritative; the runtime predicate is encapsulated in status logic and tested as behavior.
- Runtime criteria: status APIs fail nonzero in JSON and shell modes when marker blocks are malformed.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_non_block_marker_tokens`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero`
- `scripts/test-install-modes.sh`
- `.codex/scripts/task-registry source-limit check`

Full:

- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `scripts/status.sh --strict`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-strict-marker-block-validation`

Source file limit:

- Expected impact stays below 1600 lines for all touched files.
- Run `.codex/scripts/task-registry source-limit check` before task completion.

## Walkthrough Evidence

- Capture focused Rust negative test success.
- Capture shell install-mode negative test success.
- Capture full validation gates.
- Capture final `TASK_REPORT` and metrics.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-strict-marker-block-validation"

[[behaviors]]
behavior_id = "B-001-marker-block-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Ordered marker blocks pass status"
given = "Governance docs contain exact begin and end HTML comment marker lines in order"
when = "status-check runs in JSON mode"
then = "marker diagnostics pass and the command exits zero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-marker-block-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Malformed marker tokens fail status"
given = "Governance docs contain prose-only marker tokens or reversed comment markers"
when = "status-check runs in JSON mode"
then = "governance-marker diagnostics fail and the command exits nonzero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_non_block_marker_tokens"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_non_block_marker_tokens"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-shell-marker-block-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Shell strict status rejects malformed marker blocks"
given = "A temporary workspace contains markerless, prose-only, and reversed marker docs"
when = "install mode validation runs status.sh --strict"
then = "strict status rejects malformed markers before installer repair"
confirmation = "scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-marker-validation"
gap_id = "GAP-001"
polarity = "validation"
title = "Strict marker validation preserves release posture"
given = "Strict marker-block validation is implemented"
when = "source-limit, status, release, registry, Rust tests, and clippy run"
then = "all gates pass"
confirmation = ".codex/scripts/task-registry source-limit check && scripts/status.sh --strict"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && scripts/status.sh --strict"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-strict-marker-block-001"
status = "planned"
title = "Enforce exact ordered marker blocks"
kind = "implementation"
reason = "Count-only marker validation accepts prose-only and reversed marker tokens."
acceptance_proof = "Behaviors B-001-marker-block-positive and B-002-marker-block-negative pass."
behavior_ids = ["B-001-marker-block-positive", "B-002-marker-block-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "marker_check"
required_change = "Validate exact ordered HTML comment marker lines."

[[tasks.targets]]
file = "scripts/status.sh"
object = "check_markers"
required_change = "Reject prose-only tokens and end-before-begin marker order."

[[tasks]]
task_id = "TASK-2026-05-31-strict-marker-block-002"
status = "planned"
title = "Add strict marker negative tests"
kind = "validation"
reason = "Malformed marker bypasses must be locked by behavioral tests."
acceptance_proof = "Behaviors B-002-marker-block-negative and B-003-shell-marker-block-negative pass."
behavior_ids = ["B-002-marker-block-negative", "B-003-shell-marker-block-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/status_check_tests.rs"
object = "status_check_json_rejects_non_block_marker_tokens"
required_change = "Assert prose-only tokens and reversed comment markers fail JSON status."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "strict_marker_negative_cases"
required_change = "Assert shell status rejects markerless, prose-only, and reversed-marker workspaces."
```
