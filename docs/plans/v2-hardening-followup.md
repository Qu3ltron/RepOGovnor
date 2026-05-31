# V2 Hardening Follow-Up Gap Closure Contract

## Approved Scope

Close the six approved v2 hardening gaps:

1. Shell status duplicates Rust status policy.
2. Release checks do not detect required scripts omitted from `release_source.executable`.
3. Release waiver environment variables can be used without structured reason or final-release rejection.
4. Marker validation checks marker shape but not managed block content.
5. Local receipts are counted but not hash-chain verified.
6. Non-Unix executable behavior is implicit and weaker than Unix behavior.

In scope: Rust runtime diagnostics, shell rendering/delegation, migration tests, negative tests, docs, release readiness gates, source-limit-preserving module splits, and registry handoff.

Out of scope: remote signing, hosted telemetry, network receipt upload, new UI, changing marker names, changing task manifest schema, or changing existing v2 release version.

## Phased Required Change Checklist

### Phase 0: Activation and source-budget safety

- [ ] `[NEW]` `docs/plans/v2-hardening-followup.md` - `closure_contract`: declare scope, per-gap success criteria, validation, and Task Manifest.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `active_registry`: run `.codex/scripts/task-registry activate docs/plans/v2-hardening-followup.md`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `metrics_module_split`: move metrics structures and functions to a new module before adding chain behavior.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/metrics.rs` - `metrics_runtime`: own metrics parsing, formatting, and chain validation.
- [ ] `[VERIFY]` source limits - `line_budget`: run `.codex/scripts/task-registry source-limit check`.

### Phase 1: Rust-owned status policy and marker content validation

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/status_checks.rs` - `marker_check`: validate exact ordered marker block plus expected managed content signature.
- [ ] `[MODIFY]` `scripts/status.sh` - `check_markers`: delegate marker diagnostics to `.codex/scripts/task-registry status-check --format json` and render JSON instead of reimplementing marker policy.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/status_check_tests.rs` - `marker_negative_tests`: prove prose-only, reversed, duplicate, missing, and stale-content marker blocks fail.
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `status_migration_tests`: prove shell strict status rejects malformed and stale marker blocks before install repair.

### Phase 2: Release executable completeness and platform semantics

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `ReleaseCheckId`: add check IDs for undeclared executable scripts and platform executable semantics.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `release_source_executable`: fail when a required script-like release artifact is missing from `release_source.executable`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `platform_executable_semantics`: on Unix enforce mode; on non-Unix emit explicit platform diagnostics instead of silently treating file existence as executable.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `release_executable_tests`: add positive and negative release executable completeness and platform diagnostic tests without exceeding source limits.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `executable_policy_tests`: add migration fixture that removes a required script from executable policy and expects JSON failure.

### Phase 3: Waiver hardening

- [ ] `[MODIFY]` `scripts/status.sh` - `release_waivers`: require reason variables for dirty and active-task waivers; reject waivers in final release mode.
- [ ] `[MODIFY]` `scripts/release-audit.sh` - `audit_waiver`: require `AGENT_GOVERNANCE_AUDIT_TOOL_WAIVER_REASON`; reject audit waiver in final release mode.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `waiver_tests`: prove waiver-without-reason fails, reasoned local waiver passes, and final release mode rejects waivers.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Release Contract`: document waiver variables, final-release mode, and diagnostics.

### Phase 4: Receipt hash-chain integrity

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `ReceiptEvent`: add optional `previous_event_hash_sha256` and `event_hash_sha256` fields.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/receipts.rs` - `append_command_event`: compute previous hash and event hash for newly appended receipts.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `metrics_dispatch`: call the new metrics module.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/metrics.rs` - `chain_validation`: report chained, legacy unchained, malformed, and broken receipt counts.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `receipt_chain_tests`: add tests for valid chain, tampered receipt, deleted/mismatched predecessor, malformed hash, and unchained legacy receipt handling.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Receipt Event`: document chain fields and historical unchained receipt handling.

### Phase 5: Final docs, gates, and registry handoff

- [ ] `[MODIFY]` `README.md` - `v2_hardening_note`: mention reasoned waivers and Rust-owned status diagnostics if user-facing commands change.
- [ ] `[VERIFY]` focused gates - run all focused commands in the Validation Plan.
- [ ] `[VERIFY]` full gates - run format, clippy, Rust tests, release readiness, strict status, registry validation, behavior verification, and source-limit check.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `task_completion`: mark tasks completed only after linked verifiers pass; run report, metrics, and archive completed tasks if source-limit stays clean.

## Per-Gap Success Criteria

### GAP-001: Shell status duplicates Rust marker policy

- Current failure: `scripts/status.sh` owns marker parsing separately from Rust `status-check`, creating future drift risk.
- Good behavior: Given valid governance docs, when `scripts/status.sh --strict` runs, then marker pass/fail facts are rendered from Rust `status-check --format json`.
- Forbidden behavior: Shell code independently decides marker validity or disagrees with Rust JSON diagnostics.
- Files involved: `rust/task-registry-flow-cli/src/status_checks.rs`, `scripts/status.sh`, `rust/task-registry-flow-cli/src/tests/status_check_tests.rs`, `scripts/test-install-modes.sh`.
- Positive test: `scripts/status.sh --strict`.
- Negative test: `scripts/test-install-modes.sh` with stale, prose-only, reversed, duplicate, and missing marker fixtures.
- Data/schema/provenance criteria: no new marker schema; existing `governance-marker` diagnostics remain canonical.
- Runtime criteria: shell exits nonzero when Rust status JSON has marker failures.

### GAP-002: Marker blocks can be well-shaped but stale

- Current failure: a valid begin/end block with stale or wrong managed content can pass.
- Good behavior: Given the current managed marker content, status passes.
- Forbidden behavior: Given a stale marker block, wrong managed section heading, duplicate block, or missing expected content, status fails closed.
- Files involved: `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/tests/status_check_tests.rs`, `scripts/test-install-modes.sh`, `templates/AGENTS.md.template`, `templates/GEMINI.md.template`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_stale_marker_content`.
- Data/schema/provenance criteria: expected managed content is deterministic and documented.
- Runtime criteria: JSON status diagnostics include a clear stale-content actual value.

### GAP-003: Required scripts can be omitted from executable policy

- Current failure: `release_source.executable` checks listed scripts but does not fail when a new required script-like artifact is omitted.
- Good behavior: Given a required `.sh`, extensionless shebang script, or shell template release artifact, release-check requires it in executable policy where applicable.
- Forbidden behavior: A required shell script passes release-check while absent from `release_source.executable`.
- Files involved: `REQUIREMENTS.toml`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `scripts/test-release-readiness.sh`.
- Positive test: `.codex/scripts/task-registry release-check all --format json`.
- Negative test: `bash scripts/test-release-readiness.sh executable` with an executable-policy omission fixture.
- Data/schema/provenance criteria: `REQUIREMENTS.toml` remains the release policy source of truth.
- Runtime criteria: JSON diagnostics name the omitted script and check ID.

### GAP-004: Release waivers lack structured reason and final-release rejection

- Current failure: local waiver env vars can pass without a reason and without an explicit final-release fail-closed mode.
- Good behavior: Local waivers require a non-empty reason and render as notes; final-release mode rejects all waivers.
- Forbidden behavior: `AGENT_GOVERNANCE_ALLOW_* = 1` without a reason passes, or any waiver passes in final-release mode.
- Files involved: `scripts/status.sh`, `scripts/release-audit.sh`, `scripts/test-release-readiness.sh`, `docs/runtime-schemas.md`, `README.md`.
- Positive test: `bash scripts/test-release-readiness.sh status` with reasoned local waivers.
- Negative test: `bash scripts/test-release-readiness.sh waivers` for missing reasons and final-release rejection.
- Data/schema/provenance criteria: waiver reason variable names are documented.
- Runtime criteria: waiver failure messages are deterministic and grep-testable.

### GAP-005: Receipts are not hash-chain verified

- Current failure: metrics count receipts but cannot detect tampering beyond malformed JSON/schema.
- Good behavior: Newly appended receipts include previous and current hashes; metrics validates the chain.
- Forbidden behavior: Editing, reordering, deleting, or corrupting a chained receipt passes metrics as trusted.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/receipts.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/metrics.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `docs/runtime-schemas.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_rejects_tampered_receipt_chain`.
- Data/schema/provenance criteria: historical unchained v2 receipts remain readable but are reported as legacy unchained.
- Runtime criteria: chain breaks appear in metrics output and cause JSON diagnostic failure where metrics JSON is requested.

### GAP-006: Non-Unix executable behavior is implicit

- Current failure: non-Unix builds treat file existence as executable, which can mislead release diagnostics.
- Good behavior: Unix enforces executable bits; non-Unix emits explicit platform semantics rather than pretending mode enforcement happened.
- Forbidden behavior: non-Unix reports `executable file` based only on file existence.
- Files involved: `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `docs/runtime-schemas.md`, `scripts/test-release-readiness.sh`.
- Positive test: Unix chmod negative test still fails correctly.
- Negative test: Rust unit test asserts non-Unix diagnostic construction or cfg-gated expected behavior.
- Data/schema/provenance criteria: docs state platform semantics.
- Runtime criteria: release JSON is explicit about platform enforcement.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics`
- `scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh executable`
- `bash scripts/test-release-readiness.sh status`
- `bash scripts/test-release-readiness.sh waivers`
- `.codex/scripts/task-registry source-limit check`

Full:

- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `scripts/status.sh --strict`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-v2-hardening-followup`

Source file limit:

- Split `runtime.rs` before adding metrics chain logic.
- Keep all touched source, script, config, docs, and template files at or below 1600 lines.
- Run `.codex/scripts/task-registry source-limit check` before task completion.

## Walkthrough Evidence

- Rust status negative tests reject stale marker content.
- Shell install-mode migration tests reject markerless, malformed, and stale marker blocks.
- Release readiness executable tests reject executable-policy omissions.
- Waiver tests reject missing reasons and final-release waiver use.
- Metrics tests reject tampered receipt chains and report legacy unchained receipts.
- Full validation gates pass.
- Final task report and metrics are recorded.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-v2-hardening-followup"

[[behaviors]]
behavior_id = "B-001-status-rust-owned-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Shell status renders Rust status diagnostics"
given = "A valid repo with current governance marker blocks"
when = "scripts/status.sh --strict runs"
then = "status succeeds using Rust status-check diagnostics"
confirmation = "scripts/status.sh --strict"

[[behaviors.verifiers]]
type = "command"
command = "scripts/status.sh --strict"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-status-rust-owned-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Shell status rejects malformed marker migrations"
given = "Temporary workspaces contain markerless, prose-only, reversed, duplicate, and stale marker blocks"
when = "install-mode validation runs strict status"
then = "strict status fails before installer repair"
confirmation = "scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-marker-content-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Current marker content passes"
given = "Governance docs contain the current managed marker block"
when = "status-check runs in JSON mode"
then = "governance-marker diagnostics pass"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_success_exits_zero"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-marker-content-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Stale marker content fails"
given = "Governance docs have valid marker comments but stale managed content"
when = "status-check runs in JSON mode"
then = "governance-marker diagnostics fail"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_stale_marker_content"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_json_rejects_stale_marker_content"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-release-executable-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Release executable policy is complete"
given = "All required release scripts are listed in release_source.executable"
when = "release-check all runs"
then = "release executable diagnostics pass"
confirmation = ".codex/scripts/task-registry release-check all --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry release-check all --format json >/tmp/v2-hardening-release.json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-release-executable-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Undeclared release scripts fail"
given = "A required release script is omitted from release_source.executable"
when = "release readiness executable tests run"
then = "release-check fails with an undeclared executable diagnostic"
confirmation = "bash scripts/test-release-readiness.sh executable"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh executable"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007-waiver-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Reasoned local waivers are explicit"
given = "Local release status uses waiver flags with non-empty reasons"
when = "release readiness status tests run"
then = "status renders waiver notes and exits successfully"
confirmation = "bash scripts/test-release-readiness.sh status"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh status"
expected_exit = 0

[[behaviors]]
behavior_id = "B-008-waiver-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Missing or final-release waivers fail"
given = "Waiver flags lack reasons or final release mode is enabled"
when = "waiver readiness tests run"
then = "release status and audit checks fail closed"
confirmation = "bash scripts/test-release-readiness.sh waivers"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh waivers"
expected_exit = 0

[[behaviors]]
behavior_id = "B-009-receipt-chain-positive"
gap_id = "GAP-005"
polarity = "positive"
title = "Chained receipts validate"
given = "New receipts are appended through the registry runtime"
when = "metrics reads the event log"
then = "the receipt hash chain validates"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_validates_chained_receipts"
expected_exit = 0

[[behaviors]]
behavior_id = "B-010-receipt-chain-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Tampered receipt chains fail"
given = "A chained receipt is edited, reordered, or corrupted"
when = "metrics reads the event log"
then = "the chain break is reported as a failure"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_rejects_tampered_receipt_chain"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_rejects_tampered_receipt_chain"
expected_exit = 0

[[behaviors]]
behavior_id = "B-011-platform-positive"
gap_id = "GAP-006"
polarity = "positive"
title = "Platform executable semantics are explicit"
given = "Release executable checks run on the current platform"
when = "release-check all runs"
then = "the diagnostics state whether mode enforcement happened"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_reports_executable_failures"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_reports_executable_failures"
expected_exit = 0

[[behaviors]]
behavior_id = "B-012-platform-negative"
gap_id = "GAP-006"
polarity = "negative"
title = "Non-enforced executable mode is not mislabeled"
given = "Executable mode cannot be enforced on a platform"
when = "release executable diagnostics are produced"
then = "the diagnostic does not claim Unix executable-bit enforcement"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_reports_platform_executable_semantics"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_reports_platform_executable_semantics"
expected_exit = 0

[[behaviors]]
behavior_id = "B-013-v2-hardening-validation"
gap_id = "GAP-ALL"
polarity = "validation"
title = "Full v2 hardening validation passes"
given = "All hardening gaps are implemented"
when = "full repo validation runs"
then = "all source-limit, Rust, release, status, and registry gates pass"
confirmation = ".codex/scripts/task-registry source-limit check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all && scripts/status.sh --strict"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-release-readiness.sh all && scripts/status.sh --strict"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-v2-hardening-001"
status = "planned"
title = "Split metrics runtime under source limit"
kind = "implementation"
reason = "runtime.rs is near the source limit before receipt-chain behavior is added."
acceptance_proof = "Behaviors B-009-receipt-chain-positive and B-010-receipt-chain-negative pass."
behavior_ids = ["B-009-receipt-chain-positive", "B-010-receipt-chain-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "metrics_dispatch"
required_change = "Move metrics implementation to a dedicated module while preserving behavior."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "metrics_runtime"
required_change = "Own metrics report parsing, formatting, and validation."

[[tasks]]
task_id = "TASK-2026-05-31-v2-hardening-002"
status = "planned"
title = "Make Rust status diagnostics authoritative"
kind = "implementation"
reason = "Shell marker policy can drift from Rust status-check."
acceptance_proof = "Behaviors B-001-status-rust-owned-positive, B-002-status-rust-owned-negative, B-003-marker-content-positive, and B-004-marker-content-negative pass."
behavior_ids = ["B-001-status-rust-owned-positive", "B-002-status-rust-owned-negative", "B-003-marker-content-positive", "B-004-marker-content-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "marker_check"
required_change = "Validate marker shape and managed content."

[[tasks.targets]]
file = "scripts/status.sh"
object = "check_markers"
required_change = "Render Rust status-check diagnostics instead of parsing marker policy locally."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/status_check_tests.rs"
object = "marker_status_tests"
required_change = "Add stale-content and malformed-marker negative coverage."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "status_migration_tests"
required_change = "Add shell migration tests for stale marker content."

[[tasks]]
task_id = "TASK-2026-05-31-v2-hardening-003"
status = "planned"
title = "Complete release executable policy checks"
kind = "implementation"
reason = "Required scripts omitted from release_source.executable can pass release checks."
acceptance_proof = "Behaviors B-005-release-executable-positive, B-006-release-executable-negative, B-011-platform-positive, and B-012-platform-negative pass."
behavior_ids = ["B-005-release-executable-positive", "B-006-release-executable-negative", "B-011-platform-positive", "B-012-platform-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "executable_policy_checks"
required_change = "Detect undeclared script-like release artifacts and explicit platform executable semantics."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "ReleaseCheckId"
required_change = "Add release check IDs for undeclared executable scripts and platform semantics."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "release_executable_tests"
required_change = "Add positive and negative executable policy tests."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "check_executable"
required_change = "Add executable-policy omission migration fixture."

[[tasks]]
task_id = "TASK-2026-05-31-v2-hardening-004"
status = "planned"
title = "Require reasoned release waivers"
kind = "implementation"
reason = "Waiver env vars currently lack structured reason and final-release rejection."
acceptance_proof = "Behaviors B-007-waiver-positive and B-008-waiver-negative pass."
behavior_ids = ["B-007-waiver-positive", "B-008-waiver-negative"]

[[tasks.targets]]
file = "scripts/status.sh"
object = "release_waivers"
required_change = "Require waiver reasons and fail waivers in final release mode."

[[tasks.targets]]
file = "scripts/release-audit.sh"
object = "audit_waiver"
required_change = "Require audit waiver reason and fail waiver in final release mode."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "check_waivers"
required_change = "Add positive and negative waiver tests."

[[tasks]]
task_id = "TASK-2026-05-31-v2-hardening-005"
status = "planned"
title = "Add local receipt chain validation"
kind = "implementation"
reason = "Receipt metrics do not detect tampering in otherwise valid event logs."
acceptance_proof = "Behaviors B-009-receipt-chain-positive and B-010-receipt-chain-negative pass."
behavior_ids = ["B-009-receipt-chain-positive", "B-010-receipt-chain-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "ReceiptEvent"
required_change = "Add optional previous/current receipt hash fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/receipts.rs"
object = "append_command_event"
required_change = "Append hash-chained receipt events."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "chain_validation"
required_change = "Validate receipt chains and report legacy unchained receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "receipt_chain_tests"
required_change = "Add chained and tampered receipt tests."

[[tasks]]
task_id = "TASK-2026-05-31-v2-hardening-006"
status = "planned"
title = "Document v2 hardening runtime contracts"
kind = "documentation"
reason = "Changed status, release, waiver, platform, and receipt contracts must be documented."
acceptance_proof = "Behavior B-013-v2-hardening-validation passes and docs name receipt chain, reasoned waivers, and executable completeness."
behavior_ids = ["B-013-v2-hardening-validation"]

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "runtime_contracts"
required_change = "Document status, release executable completeness, waiver, platform, and receipt-chain behavior."

[[tasks.targets]]
file = "README.md"
object = "v2_hardening_note"
required_change = "Document user-facing waiver/status behavior changes."
```
