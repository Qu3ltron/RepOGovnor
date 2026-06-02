# Cost Ingest Nix Git Dependency Gap Closure Contract

## Approved Scope

Fix the release package gate for cost ingestion by declaring the `git`
executable in the Nix package test/build environment. `cost-ingest` resolves
explicit commit targets through `git rev-parse`; package checks must exercise
that path instead of silently depending on the developer shell.

In scope:

- Add the Nix package dependency needed for cost-ingest tests that execute
  local Git commands.
- Re-run the package/release gates that exposed the failure.

Out of scope:

- Changing cost-ingest commit attribution semantics.
- Adding compatibility shims or bypassing commit resolution in tests.

## Phased Required Change Checklist

### Phase 0: Activation

- [ ] `[NEW]` `docs/plans/cost-ingest-nix-git-dependency-2026-06-02.md` - `closure_contract`: create this follow-up contract.
- [ ] `[VERIFY]` `docs/plans/cost-ingest-nix-git-dependency-2026-06-02.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/cost-ingest-nix-git-dependency-2026-06-02.md`.

### Phase 1: Package gate fix

- [ ] `[MODIFY]` `package.nix` - `native_build_inputs`: include `git` for package tests that resolve commits.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs` - `concurrent_writer_retry_budget`: make lock-contention retry budget adequate for slow Nix package sandboxes.

### Phase 2: Validation

- [ ] `[VERIFY]` `package.nix` - `nix_package`: `nix build .#task-registry-flow`.
- [ ] `[VERIFY]` `scripts/test-release-readiness.sh` - `release_readiness`: `bash scripts/test-release-readiness.sh all`.

## Per-Gap Success Criteria

### GAP-001: Nix package tests do not include the Git executable

- Current failure: `bash scripts/test-release-readiness.sh all` reaches the Nix
  package build, but cost-ingest tests panic because `git` is not found in the
  package test environment.
- Good behavior: Given the package build environment, when package tests run,
  then cost-ingest tests can create a real local commit and exercise commit
  resolution.
- Forbidden behavior: Do not bypass commit resolution or replace Git-backed
  commit attribution with a synthetic test-only path.
- Files involved: `package.nix`, `rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs`.
- Positive test: `nix build .#task-registry-flow`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture`.
- Data/schema/provenance: Commit resolution remains `git rev-parse`.
- Runtime: No remote network or push.

## Validation Plan

Focused:

- `nix build .#task-registry-flow`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture`

Full:

- `bash scripts/test-release-readiness.sh all`

## Source File Limit

All touched files remain under 1600 lines.

## Walkthrough Evidence

- Package build failure showed `git` missing during cost-ingest tests.
- `nix build .#task-registry-flow` passes after the dependency is declared.
- Release readiness passes after the package build passes.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-cost-ingest-nix-git-dependency"

[[behaviors]]
behavior_id = "B-001-nix-package-git-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Nix package tests can resolve Git commits"
given = "The Nix package test environment declares the Git executable"
when = "the task-registry-flow package builds"
then = "cost-ingest tests can create a real commit and resolve it"
confirmation = "nix build .#task-registry-flow"

[[behaviors.verifiers]]
type = "command"
command = "nix build .#task-registry-flow"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-commit-required-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Commit attribution remains mandatory"
given = "Cost ingestion is attempted without an explicit commit"
when = "the command parser runs"
then = "the runtime rejects the request instead of bypassing commit attribution"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-release-readiness-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Release readiness includes package gate"
given = "The Nix package dependency is declared"
when = "release readiness runs"
then = "the package and release gates pass"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-cost-ingest-nix-git-dependency-001"
behavior_ids = [
  "B-001-nix-package-git-positive",
  "B-002-commit-required-negative",
  "B-003-release-readiness-validation",
]
status = "planned"
title = "Declare Git for Nix package cost-ingest tests"
kind = "release"
reason = "Cost-ingest commit attribution uses real Git commit resolution, so package tests need Git available."
acceptance_proof = "Behaviors B-001, B-002, and B-003."

[[tasks.targets]]
file = "package.nix"
object = "native_build_inputs"
required_change = "Include git for package tests that resolve commits."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/receipt_chain_tests.rs"
object = "concurrent_writer_retry_budget"
required_change = "Make receipt lock contention retry budget adequate for slow Nix package sandboxes."

[[tasks.targets]]
file = "docs/plans/cost-ingest-nix-git-dependency-2026-06-02.md"
object = "closure_contract"
required_change = "Track the follow-up package-gate gap and validation evidence."

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "active_registry"
required_change = "Record activated package dependency task."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "release_gate_receipts"
required_change = "Append activation, landing, and validation receipts."
```
