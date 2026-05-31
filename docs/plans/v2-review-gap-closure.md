# V2 Review Gap Closure Contract

## Approved Scope

Close the two review-blocking gaps surfaced after the v2 hardening implementation:

1. `scripts/status.sh` calls `cargo run` directly for `status-check`, bypassing the canonical task-registry wrapper and its safe `CARGO_TARGET_DIR`.
2. New metrics Rust source files are not release-governed, so release checks can omit them from `REQUIREMENTS.toml`.

In scope: canonical status-check invocation, inherited unwritable target-dir regression coverage, release-source manifest completeness, Rust release-check hardening for undeclared Rust sources, tests, docs, and registry handoff.

Out of scope: release version changes, compatibility shims, hosted signing, remote telemetry, unrelated release artifact reshaping, and changing task manifest schema.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/v2-review-gap-closure.md` - `closure_contract`: declare approved scope, per-gap success criteria, validation plan, and Task Manifest.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `active_registry`: run `.codex/scripts/task-registry activate docs/plans/v2-review-gap-closure.md`.
- [ ] `[VERIFY]` source limits - `line_budget`: run `.codex/scripts/task-registry source-limit check`.

### Phase 1: Canonical status-check execution

- [ ] `[MODIFY]` `scripts/status.sh` - `task_registry`: route status-check and release-source CLI calls through `${TARGET_ROOT}/.codex/scripts/task-registry` when present, with the same safe target-dir fallback used by the wrapper when absent.
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `status_target_dir_regression`: prove inherited unwritable `CARGO_TARGET_DIR` still yields Rust marker diagnostics instead of `missing status diagnostic`.

### Phase 2: Release-source manifest completeness

- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: add `rust/task-registry-flow-cli/src/metrics.rs`, `rust/task-registry-flow-cli/src/tests/metrics_tests.rs`, and any currently omitted Rust source file surfaced by the completeness check.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.check_ids`: add the Rust-source completeness check ID.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `ReleaseCheckId`: add canonical `release-rust-source-undeclared`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `rust_source_policy_checks`: fail when a `rust/task-registry-flow-cli/src/**/*.rs` file is absent from `release_source.required`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `release_source_negative`: add a negative unit test for an undeclared Rust source.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `release_manifest_assertions`: assert metrics sources are present in release-check output and that omitting one fails closed.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Release Contract`: document Rust source completeness diagnostics.

### Phase 3: Validation and handoff

- [ ] `[VERIFY]` focused gates - run all focused commands in the Validation Plan.
- [ ] `[VERIFY]` full gates - run format, clippy, Rust tests, release readiness, strict status, registry validation, behavior verification, and source-limit check.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `task_completion`: mark tasks completed only after linked verifiers pass; run report and metrics.

## Per-Gap Success Criteria

### GAP-001: Status-check bypasses canonical cargo target handling

- Current failure: `scripts/status.sh` runs `cargo run` directly for `status-check`, so an inherited unwritable `CARGO_TARGET_DIR` can prevent JSON diagnostics and produce `missing status diagnostic`.
- Good behavior: Given valid or invalid marker blocks, when `scripts/status.sh --strict` runs with an inherited unwritable `CARGO_TARGET_DIR`, then it still obtains Rust status-check JSON through the canonical wrapper path or equivalent safe fallback.
- Forbidden behavior: Shell status falls back to empty JSON, reports `missing status diagnostic`, or depends on inherited `CARGO_TARGET_DIR`.
- Files involved: `scripts/status.sh`, `scripts/test-install-modes.sh`.
- Positive test: `scripts/status.sh --strict`.
- Negative test: `scripts/test-install-modes.sh`.
- Data/schema/provenance criteria: no new status schema; `status-check --format json` remains canonical.
- Runtime criteria: target-dir handling uses `.codex/scripts/task-registry` when available and otherwise sets `CARGO_TARGET_DIR=${AGENT_GOVERNANCE_CARGO_TARGET_DIR:-/tmp/agent-governance-cargo-target}`.

### GAP-002: Metrics sources are not release-governed

- Current failure: `rust/task-registry-flow-cli/src/metrics.rs` and `rust/task-registry-flow-cli/src/tests/metrics_tests.rs` exist but are omitted from `release_source.required`; a full source sweep also surfaces any earlier Rust source omissions, and release checks do not catch undeclared Rust source files.
- Good behavior: Given all Rust task-registry source files, when `release-check required|all` runs, then every `rust/task-registry-flow-cli/src/**/*.rs` file is either listed in `release_source.required` or the release check fails with `release-rust-source-undeclared`.
- Forbidden behavior: Adding or omitting a Rust source file from `release_source.required` still allows release checks to pass.
- Files involved: `REQUIREMENTS.toml`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `scripts/test-release-readiness.sh`, `docs/runtime-schemas.md`.
- Positive test: `bash scripts/test-release-readiness.sh executable`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_rejects_undeclared_rust_sources`.
- Data/schema/provenance criteria: `REQUIREMENTS.toml` remains the release policy source of truth and names the new check ID.
- Runtime criteria: JSON diagnostics name the undeclared Rust source path and remediation.

## Validation Plan

Focused:

- `scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh executable`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_rejects_undeclared_rust_sources`
- `.codex/scripts/task-registry release-check all --format json`
- `.codex/scripts/task-registry source-limit check`

Full:

- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`
- `scripts/status.sh --strict`
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-v2-review-gap-closure`

Source file limit:

- Keep all touched source, script, config, docs, and governance files at or below 1600 lines.
- Run `.codex/scripts/task-registry source-limit check` before task completion.

## Walkthrough Evidence

- Install-mode tests prove inherited bad `CARGO_TARGET_DIR` still renders marker-specific diagnostics.
- Release readiness executable tests prove metrics files are release-listed and omission fails closed.
- Rust release schema tests reject undeclared Rust source files.
- Full validation gates pass.
- Final task report and metrics are recorded.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-v2-review-gap-closure"

[[behaviors]]
behavior_id = "B-001-status-canonical-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Strict status uses canonical status-check execution"
given = "A valid repository with current governance marker blocks"
when = "scripts/status.sh --strict runs"
then = "status succeeds using Rust status-check diagnostics without bypassing canonical target-dir handling"
confirmation = "scripts/status.sh --strict"

[[behaviors.verifiers]]
type = "command"
command = "scripts/status.sh --strict"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-status-target-dir-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Inherited unwritable cargo target does not hide marker diagnostics"
given = "A temporary install workspace with marker failures and an inherited unwritable CARGO_TARGET_DIR"
when = "strict status runs before installer repair"
then = "status fails with marker-specific diagnostics, not missing status diagnostic"
confirmation = "scripts/test-install-modes.sh"

[[behaviors.verifiers]]
type = "command"
command = "scripts/test-install-modes.sh"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-release-source-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Metrics Rust sources are release-governed"
given = "The current release manifest includes every Rust task-registry source file"
when = "release readiness executable checks run"
then = "release-check reports metrics.rs and metrics_tests.rs as required release files"
confirmation = "bash scripts/test-release-readiness.sh executable"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh executable"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-release-source-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Undeclared Rust source fails release checks"
given = "A release fixture contains a Rust source file absent from release_source.required"
when = "release-check required runs"
then = "release-check fails with release-rust-source-undeclared"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_rejects_undeclared_rust_sources"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_rejects_undeclared_rust_sources"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-review-validation"
gap_id = "VALIDATION"
polarity = "validation"
title = "Review gap closure full validation passes"
given = "Implementation gaps are closed"
when = "full validation runs"
then = "format, tests, release readiness, strict status, registry validation, and source limits pass"
confirmation = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check"
expected_exit = 0

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry validate"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-v2-review-001"
status = "planned"
behavior_ids = ["B-001-status-canonical-positive", "B-002-status-target-dir-negative"]
title = "Restore canonical status-check target handling"
kind = "implementation"
reason = "Close review gap where strict status bypassed the task-registry wrapper target-dir contract."
acceptance_proof = "Behaviors B-001 and B-002: scripts/status.sh --strict; scripts/test-install-modes.sh"

[[tasks.targets]]
file = "scripts/status.sh"
object = "task_registry"
required_change = "Route status-check and internal release-source CLI calls through canonical task-registry execution with safe target-dir fallback."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "status_target_dir_regression"
required_change = "Add an inherited unwritable CARGO_TARGET_DIR regression that expects marker-specific diagnostics."

[[tasks]]
task_id = "TASK-2026-05-31-v2-review-002"
status = "planned"
behavior_ids = ["B-003-release-source-positive", "B-004-release-source-negative"]
title = "Govern metrics Rust sources in release checks"
kind = "release"
reason = "Close review gap where new metrics source files could be omitted from release-source governance."
acceptance_proof = "Behaviors B-003 and B-004: bash scripts/test-release-readiness.sh executable; cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml release_schema_rejects_undeclared_rust_sources"

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Add metrics.rs, metrics_tests.rs, and any currently omitted Rust source file surfaced by the completeness check to required release artifacts."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.check_ids"
required_change = "Declare release-rust-source-undeclared as a known release check."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "ReleaseCheckId"
required_change = "Add release-rust-source-undeclared canonical check ID."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "rust_source_policy_checks"
required_change = "Fail release checks when Rust source files under src are absent from release_source.required."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "release_schema_rejects_undeclared_rust_sources"
required_change = "Add negative coverage for undeclared Rust source files."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "release_manifest_assertions"
required_change = "Assert metrics source files are release-listed and omission fails closed."

[[tasks]]
task_id = "TASK-2026-05-31-v2-review-003"
status = "planned"
behavior_ids = ["B-003-release-source-positive", "B-005-review-validation"]
title = "Document release-source completeness and validate"
kind = "documentation"
reason = "Keep runtime schema docs aligned with the reinforced release-source contract."
acceptance_proof = "Behaviors B-003 and B-005: bash scripts/test-release-readiness.sh executable; .codex/scripts/task-registry source-limit check && .codex/scripts/task-registry validate"

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "Release Contract"
required_change = "Document Rust source completeness and release-rust-source-undeclared diagnostics."

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "task_completion"
required_change = "Record completion after focused and full validation pass."
```
