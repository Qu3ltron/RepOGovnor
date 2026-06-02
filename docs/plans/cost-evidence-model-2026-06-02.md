# Cost Evidence Model Gap Closure Contract

## Approved Scope

Close the next GP-007 slice by adding typed cost evidence receipts and a local
diagnostic command that classifies token/cost evidence as `measured`,
`estimated`, or `unmeasured`.

In scope:

- Provider-neutral receipt schema for cost evidence.
- `cost-evidence-check [--format json]` validation.
- Local metrics counts for measured, estimated, and unmeasured cost evidence.
- Public docs that keep cost-per-commit unshipped unless commit-linked measured
  usage receipts exist.

Out of scope:

- Collecting provider usage automatically.
- Calculating cost per commit.
- Pricing lookups, remote telemetry, hosted dashboards, or non-Codex adapters.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/cost-evidence-model-2026-06-02.md` - `closure_contract`: create and activate this contract before implementation.
- [ ] `[VERIFY]` `docs/plans/cost-evidence-model-2026-06-02.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/cost-evidence-model-2026-06-02.md`.

### Phase 1: Runtime cost evidence

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `cost_evidence_schema`: add cost evidence status, target, usage, pricing, amount, and receipt fields.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - `metrics_report`: add cost evidence count fields.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/metrics.rs` - `cost_counts`: count measured, estimated, and unmeasured cost evidence receipts.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_evidence.rs` - `cost_evidence_check`: validate honest cost evidence classification.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module_list`: include the cost evidence module.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: add `cost-evidence-check [--format json]`.

### Phase 2: Tests and release docs

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `cost_evidence_tests`: prove measured evidence passes, estimated/unmeasured are explicit, and false measured evidence fails.
- [ ] `[MODIFY]` `README.md` - `cost_evidence`: document the new command and non-claim boundaries.
- [ ] `[MODIFY]` `ROADMAP.md` - `cost_evidence`: record typed cost evidence as the current slice before cost per commit.
- [ ] `[MODIFY]` `docs/engineering-policy-compliance.md` - `token_cost_evidence`: describe current measured/estimated/unmeasured schema posture.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `cost_evidence_schema`: document receipt fields, validation, and metrics.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-007`: update current evidence and remaining reactivation conditions.
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `v2_1_0`: include this plan in the release evidence list.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source`: include the new runtime module.

## Per-Gap Success Criteria

### GAP-001: Cost evidence lacks typed measured, estimated, and unmeasured states

- Current failure: token/cost posture is prose only; receipts cannot carry typed cost evidence.
- Good behavior: Given receipts with cost evidence, when `cost-evidence-check --format json` runs, then measured receipts pass only with provider, model, usage, pricing, timestamp, target, evidence source, and amount evidence.
- Forbidden behavior: A receipt marked `measured` without required usage or pricing evidence must fail diagnostics.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_check_reports_measured_estimated_and_unmeasured -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_check_rejects_false_measured_receipt -- --nocapture`
- Domain/API/UI: Adds `cost-evidence-check [--format json]`.
- Runtime: Local JSONL receipts only; no remote telemetry.

### GAP-002: Cost metrics can be overread as cost-per-commit readiness

- Current failure: model attribution and workflow metrics could be mistaken for spend evidence.
- Good behavior: Metrics count cost evidence state separately, and docs say cost per commit remains unmeasured until commit-linked measured usage receipts exist.
- Forbidden behavior: Docs must not claim reliable cost per commit, measured token spend, or automatic pricing collection today.
- Files involved: `rust/task-registry-flow-cli/src/model.rs`, `rust/task-registry-flow-cli/src/metrics.rs`, `README.md`, `ROADMAP.md`, `docs/engineering-policy-compliance.md`, `docs/runtime-schemas.md`, `docs/gap-pipeline.md`.
- Positive test: `rg -n 'cost-evidence-check|measured, estimated, and unmeasured|commit-linked measured usage receipts' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md`
- Negative test: `! rg -n 'reliable cost per commit today|automatic pricing collection|measured token spend today' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md`
- Domain/API/UI: Cost metrics are evidence state counts, not calculated spend.
- Runtime: `metrics` reports state counts only.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_runtime_surface_tests -- --nocapture`
- `.codex/scripts/task-registry cost-evidence-check --format json`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml --all-targets -- -D warnings`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected line-budget impact is small. `schema.rs`, `runtime.rs`, docs, and the
new module remain below 1600 lines. `tests/mod.rs` is already at the limit, so
new tests must go into an existing included test module without adding a module
declaration.

## Walkthrough Evidence

- Focused and full Rust test output.
- Cost evidence diagnostic output.
- Landing verification for the exact changed files.
- Registry report, metrics, source-limit, validation, and receipt-chain output.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-cost-evidence-model"

[[behaviors]]
behavior_id = "B-001-cost-evidence-measured-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Measured, estimated, and unmeasured cost evidence is classified"
given = "Local receipts contain measured, estimated, and unmeasured cost evidence"
when = "cost-evidence-check runs"
then = "measured evidence passes and estimated/unmeasured evidence is explicitly classified"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_check_reports_measured_estimated_and_unmeasured -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_check_reports_measured_estimated_and_unmeasured -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-cost-evidence-measured-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "False measured cost evidence fails closed"
given = "A receipt claims measured cost evidence without required usage and pricing fields"
when = "cost-evidence-check runs"
then = "the diagnostic report fails the receipt"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_check_rejects_false_measured_receipt -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_check_rejects_false_measured_receipt -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-cost-evidence-docs-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Docs expose cost evidence without claiming cost per commit"
given = "Public docs describe token and cost evidence"
when = "the docs are inspected"
then = "they describe cost-evidence-check and commit-linked measured usage receipt requirements"
confirmation = "rg -n 'cost-evidence-check|measured, estimated, and unmeasured|commit-linked measured usage receipts' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'cost-evidence-check|measured, estimated, and unmeasured|commit-linked measured usage receipts' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-cost-evidence-docs-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Docs reject unsupported spend claims"
given = "Public docs describe current cost posture"
when = "unsupported spend claims are searched"
then = "the docs do not claim reliable cost per commit today, automatic pricing collection, or measured token spend today"
confirmation = "! rg -n 'reliable cost per commit today|automatic pricing collection|measured token spend today' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'reliable cost per commit today|automatic pricing collection|measured token spend today' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-cost-evidence-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Cost evidence release gates pass"
given = "The cost evidence implementation is present"
when = "focused validation runs"
then = "source limit and runtime surface tests pass"
confirmation = ".codex/scripts/task-registry source-limit check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_runtime_surface_tests -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_runtime_surface_tests -- --nocapture"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-cost-evidence-model-001"
behavior_ids = [
  "B-001-cost-evidence-measured-positive",
  "B-002-cost-evidence-measured-negative",
]
status = "planned"
title = "Add typed cost evidence runtime validation"
kind = "implementation"
reason = "GP-007 needs structured usage evidence states before any honest spend metric."
acceptance_proof = "Behaviors B-001-cost-evidence-measured-positive and B-002-cost-evidence-measured-negative."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "cost_evidence_schema"
required_change = "Add provider-neutral cost evidence types and optional receipt field."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_evidence.rs"
object = "cost_evidence_check"
required_change = "Validate measured, estimated, and unmeasured cost evidence honestly."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module_list"
required_change = "Include the cost evidence module."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Dispatch cost-evidence-check."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "cost_evidence_tests"
required_change = "Add focused positive and negative cost evidence tests."

[[tasks]]
task_id = "TASK-2026-06-02-cost-evidence-model-002"
behavior_ids = [
  "B-003-cost-evidence-docs-positive",
  "B-004-cost-evidence-docs-negative",
  "B-005-cost-evidence-validation",
]
status = "planned"
title = "Expose honest cost evidence docs and metrics"
kind = "governance"
reason = "Cost evidence state counts must not be overread as calculated spend."
acceptance_proof = "Behaviors B-003-cost-evidence-docs-positive, B-004-cost-evidence-docs-negative, and B-005-cost-evidence-validation."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "metrics_report"
required_change = "Add measured, estimated, and unmeasured cost evidence counts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "cost_counts"
required_change = "Count cost evidence receipts by status."

[[tasks.targets]]
file = "README.md"
object = "cost_evidence"
required_change = "Document cost-evidence-check and current non-claims."

[[tasks.targets]]
file = "ROADMAP.md"
object = "cost_evidence"
required_change = "Reflect typed cost evidence as the next slice before cost per commit."

[[tasks.targets]]
file = "docs/engineering-policy-compliance.md"
object = "token_cost_evidence"
required_change = "Document current measured, estimated, and unmeasured cost evidence posture."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "cost_evidence_schema"
required_change = "Document cost evidence receipt schema, command, and metrics."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-007"
required_change = "Update current evidence and remaining cost-per-commit gap."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "v2_1_0"
required_change = "Add this plan to release evidence."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source"
required_change = "Include the new cost evidence module."
```
