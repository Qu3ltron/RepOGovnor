# Codex Model Attribution Hardening Gap Closure Contract

## Approved Scope

Add first-class model attribution for hook-observed repo mutation attempts,
starting with Codex because Codex hook payloads expose the active `model` slug.
The implementation must stay provider-neutral: Codex is the first measured
adapter, not the product boundary.

In scope:
- Runtime schema fields for provider-neutral model and mutation attribution.
- Codex PreToolUse fail-closed model identity enforcement for supported writes.
- Codex PostToolUse outcome receipt recording where hook data is available.
- Model attribution diagnostics, metrics, reviewer-report summary, docs, hooks,
  release-source coverage, and tests.

Out of scope:
- Token usage, pricing, or cost-per-commit calculation.
- Transcript parsing as enforcement evidence.
- Regulatory compliance, external attestation, or non-Codex measured adapters.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/codex-model-attribution-hardening-2026-06-02.md` - `closure_contract`: create and activate this contract before implementation.
- [ ] `[VERIFY]` `docs/plans/codex-model-attribution-hardening-2026-06-02.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/codex-model-attribution-hardening-2026-06-02.md`.

### Phase 1: Runtime model attribution
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `receipt_schema`: add provider-neutral attribution enums/structs, report surface, and command.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - `metrics_report`: add measured/unmeasured mutation attribution counts.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module_list`: include the model attribution module.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/model_attribution.rs` - `model_attribution_check`: scan local receipts and report attribution posture.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: add `model-attribution-check [--format json]`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/mutation_hook.rs` - `codex_attribution`: enforce Codex PreToolUse model identity for supported writes and append attribution receipts.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/metrics.rs` - `attribution_counts`: count measured and unmeasured mutation attribution receipts.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/reviewer_report.rs` - `attribution_summary`: include measured/unmeasured model attribution counts.

### Phase 2: Hooks, docs, release
- [MODIFY] `.codex/hooks.json` - `codex_posttooluse`: add Codex PostToolUse model attribution recording hook.
- [MODIFY] `hooks/codex-hooks.json` - `codex_posttooluse`: mirror packaged Codex hook.
- [MODIFY] `templates/.codex/hooks.json.template` - `codex_posttooluse`: project PostToolUse hook projection.
- [MODIFY] `README.md` - `model_attribution`: document Codex as first measured adapter and non-Codex as unmeasured.
- [MODIFY] `ROADMAP.md` - `cost_evidence`: reflect shipped model attribution before token/cost.
- [MODIFY] `docs/engineering-policy-compliance.md` - `model_evidence`: clarify model identity versus spend evidence.
- [MODIFY] `docs/runtime-schemas.md` - `receipt_schema`: document attribution fields and `model-attribution-check`.
- [MODIFY] `docs/gap-pipeline.md` - `cost_gap`: update current evidence and remaining gap.
- [MODIFY] `docs/version-roadmap.toml` - `covered_plan_ids`: cover this completed plan in 2.1.0.
- [MODIFY] `REQUIREMENTS.toml` - `release_source.required`: include new Rust module and test.

### Phase 3: Tests and landing
- [NEW] `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `model_attribution_tests`: prove Codex measured attribution, missing model denial, PostToolUse recording, and non-Codex unmeasured posture.
- [MODIFY] `rust/task-registry-flow-cli/src/tests/mod.rs` - `test_module_list`: include model attribution tests without exceeding source limit.
- [VERIFY] `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `focused_tests`: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture`.
- [VERIFY] `source_limit`: `.codex/scripts/task-registry source-limit check`.
- [VERIFY] `registry_validation`: `.codex/scripts/task-registry validate`.
- [VERIFY] `release_posture`: `.codex/scripts/task-registry version-check validate && .codex/scripts/task-registry backlog-check`.

## Per-Gap Success Criteria

### GAP-001: Codex mutation governance does not know which model requested the mutation
- Current failure: Codex mutation hooks validate target scope but do not require or record active model identity.
- Good behavior: Given a Codex PreToolUse write payload with `model`, `session_id`, `turn_id`, and `tool_use_id`, when the target is active, then the hook passes and records measured model attribution.
- Forbidden behavior: Given a Codex PreToolUse write payload missing `model` or `tool_use_id`, when the target is active, then the hook fails closed before allowing mutation.
- Files involved: `rust/task-registry-flow-cli/src/mutation_hook.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture`.
- Domain/API/UI: Runtime receipt schema gains optional attribution objects.
- Runtime: Existing mutation-scope denial behavior remains intact.

### GAP-002: Model attribution is not queryable as compliance evidence
- Current failure: There is no command or report surface that distinguishes measured from unmeasured mutation attribution.
- Good behavior: Given local mutation attribution receipts, when `model-attribution-check --format json` runs, then Codex measured receipts pass and non-Codex unmeasured receipts are reported honestly.
- Forbidden behavior: Non-Codex or missing data must not be guessed as measured model identity.
- Files involved: `rust/task-registry-flow-cli/src/model_attribution.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/metrics.rs`, `rust/task-registry-flow-cli/src/reviewer_report.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture`.
- Domain/API/UI: Adds `model-attribution-check [--format json]`.
- Runtime: Metrics and reviewer report include attribution counts.

### GAP-003: Public docs can overread Codex support as universal cost attribution
- Current failure: Cost direction is documented, but model responsibility for mutation needs a shipped first step and clear non-claims.
- Good behavior: Docs state Codex is the first measured adapter, other surfaces are unmeasured until implemented, and model identity is not token spend.
- Forbidden behavior: Docs must not claim reliable cost per commit, token usage, or universal model attribution.
- Files involved: `README.md`, `ROADMAP.md`, `docs/engineering-policy-compliance.md`, `docs/runtime-schemas.md`, `docs/gap-pipeline.md`, `docs/version-roadmap.toml`, `REQUIREMENTS.toml`.
- Positive test: `rg -n 'model-attribution-check|Codex.*first measured|unmeasured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md`.
- Negative test: `! rg -n 'reliable cost per commit today|universal model attribution|token spend is measured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md`.
- Domain/API/UI: Docs and release-source coverage only.
- Runtime: Version and backlog checks remain green.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml typed_runtime_surface_tests -- --nocapture`
- `rg -n 'model-attribution-check|Codex.*first measured|unmeasured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md`
- `! rg -n 'reliable cost per commit today|universal model attribution|token spend is measured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md`
- `.codex/scripts/task-registry source-limit check`
- `.codex/scripts/task-registry validate`

Full:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `.codex/scripts/task-registry version-check validate`
- `.codex/scripts/task-registry backlog-check`
- `.codex/scripts/task-registry status-check --format json`
- `.codex/scripts/task-registry verify-chain --format json`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

`rust/task-registry-flow-cli/src/tests/mod.rs` is near the limit, so only one
module declaration may be added there. New test behavior belongs in
`rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`. Final
verification includes `.codex/scripts/task-registry source-limit check`.

## Walkthrough Evidence

- Plan activation output.
- Focused model attribution tests.
- Source-limit and registry validation output.
- Version and backlog checks.
- Full Rust test suite and release-readiness output.
- `TASK_REPORT` and `TASK_METRICS` after landing.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-codex-model-attribution-hardening"

[[behaviors]]
behavior_id = "B-001-codex-attribution-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Codex write records measured model attribution"
given = "A Codex PreToolUse write payload includes model, session, turn, tool use, and an active target"
when = "verify-mutation-hook evaluates the payload"
then = "the hook passes and records a measured attribution receipt"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-codex-attribution-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Codex write missing model identity fails closed"
given = "A Codex PreToolUse write payload omits required model attribution"
when = "verify-mutation-hook evaluates the payload"
then = "the hook denies the mutation with a model attribution failure"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-model-check-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Model attribution check reports measured and unmeasured posture"
given = "Local receipts contain Codex measured attribution and non-Codex unmeasured attribution"
when = "model-attribution-check --format json runs"
then = "the diagnostic report exposes measured and unmeasured mutation attribution honestly"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-model-check-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Model attribution check rejects bad measured Codex receipts"
given = "A Codex mutation attribution receipt is marked measured without required model evidence"
when = "model-attribution-check --format json runs"
then = "the diagnostic report fails instead of accepting false precision"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml model_attribution -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-docs-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Docs state Codex first measured adapter and unmeasured non-Codex posture"
given = "Public docs describe model attribution"
when = "The docs are searched"
then = "They state model-attribution-check, Codex as first measured adapter, and unmeasured gaps"
confirmation = "rg -n 'model-attribution-check|Codex.*first measured|unmeasured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'model-attribution-check|Codex.*first measured|unmeasured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-docs-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Docs avoid token spend and universal attribution overclaims"
given = "Public docs describe model attribution"
when = "Overclaim phrases are searched"
then = "The docs do not claim reliable cost per commit today, universal model attribution, or measured token spend"
confirmation = "! rg -n 'reliable cost per commit today|universal model attribution|token spend is measured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'reliable cost per commit today|universal model attribution|token spend is measured' README.md ROADMAP.md docs/engineering-policy-compliance.md docs/runtime-schemas.md docs/gap-pipeline.md"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-codex-model-attribution-hardening-001"
title = "Implement runtime model attribution"
status = "planned"
kind = "implementation"
reason = "Codex mutation governance must record which active model is responsible for supported repo mutation attempts."
acceptance_proof = "Behaviors B-001, B-002, B-003, and B-004 pass their typed verifiers."
behavior_ids = ["B-001-codex-attribution-positive", "B-002-codex-attribution-negative", "B-003-model-check-positive", "B-004-model-check-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "receipt_schema"
required_change = "Add attribution enums, structs, report surface, and command."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "metrics_report"
required_change = "Add model attribution metrics counts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module_list"
required_change = "Include the model attribution module."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model_attribution.rs"
object = "model_attribution_check"
required_change = "Implement receipt scanning and diagnostics for model attribution."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Route model-attribution-check and update usage."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "codex_attribution"
required_change = "Require Codex PreToolUse model identity for supported writes and append attribution receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "attribution_counts"
required_change = "Count measured and unmeasured mutation attribution receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reviewer_report.rs"
object = "attribution_summary"
required_change = "Show model attribution counts in reviewer reports."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "model_attribution_tests"
required_change = "Add focused attribution runtime tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "test_module_list"
required_change = "Include model attribution tests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/hook_command_tests.rs"
object = "codex_identity_fixture"
required_change = "Carry required Codex identity fields in existing write-scope hook command fixtures."

[[tasks]]
task_id = "TASK-2026-06-02-codex-model-attribution-hardening-002"
title = "Document and package model attribution hardening"
status = "planned"
kind = "documentation"
reason = "Public posture must state Codex as first measured adapter without claiming token spend or universal attribution."
acceptance_proof = "Behaviors B-005 and B-006 pass their typed verifiers."
behavior_ids = ["B-005-docs-positive", "B-006-docs-negative"]

[[tasks.targets]]
file = ".codex/hooks.json"
object = "codex_posttooluse"
required_change = "Add Codex PostToolUse model attribution hook."

[[tasks.targets]]
file = "hooks/codex-hooks.json"
object = "codex_posttooluse"
required_change = "Mirror packaged Codex PostToolUse hook."

[[tasks.targets]]
file = "templates/.codex/hooks.json.template"
object = "codex_posttooluse"
required_change = "Project PostToolUse hook projection."

[[tasks.targets]]
file = "README.md"
object = "model_attribution"
required_change = "Document Codex as first measured adapter and non-Codex as unmeasured."

[[tasks.targets]]
file = "ROADMAP.md"
object = "cost_evidence"
required_change = "Reflect shipped model attribution before token/cost evidence."

[[tasks.targets]]
file = "docs/engineering-policy-compliance.md"
object = "model_evidence"
required_change = "Clarify model identity versus token spend evidence."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "receipt_schema"
required_change = "Document attribution fields and model-attribution-check."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "cost_gap"
required_change = "Update current evidence and remaining cost gap."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "covered_plan_ids"
required_change = "Add this plan to 2.1.0 release coverage."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Include model attribution module and tests."
```
