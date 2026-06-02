# Token Capture Review Gap Closure Contract

## Approved Scope

Close the four P2 review gaps in token-capture behavior:

- Service-tier-specific measured receipts must not dedupe each other.
- Named `[[service_tiers]]` rates must win over stale top-level `models` when the selected tier has a matching name.
- Reasoning-token pricing must fail closed unless the snapshot policy is explicitly supported.
- Cost coverage for tool-bound mutations must require matching tool-bound cost evidence.

Out of scope: compatibility shims, new provider adapters, new public receipt schema fields, automatic pushes, or migration of unrelated historical receipts.

## Phased Required Change Checklist

### Phase 0: Activation and safety
- [ ] `[NEW]` `docs/plans/token-capture-review-gap-closure-2026-06-02.md` - `closure contract`: create schema v2 manifest with positive and negative behavior coverage for every review gap; acceptance proof is `PLAN_ACTIVATE`.
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `repo boundary`: run activation from `/home/hasnamuss/reclaimed/work/RepOGovnor`; acceptance proof is `git rev-parse --show-toplevel`.
- [ ] `[VERIFY]` `.codex/scripts/task-registry` - `source file limit`: run source-limit before completion; acceptance proof is `.codex/scripts/task-registry source-limit check`.

### Phase 1: Service tier receipt identity
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_ingest.rs` - `receipt_exists`: include selected service tier, pricing snapshot path/hash, source/version/currency, rates, and amount in the measured receipt duplicate key; acceptance proof is `cost_ingest_appends_distinct_receipts_per_service_tier` and `cost_ingest_dedupes_only_identical_pricing_identity`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_report.rs` - `report grouping key`: include service tier in measured report grouping so same target/model/version across tiers is not merged; acceptance proof is `cost_ingest_appends_distinct_receipts_per_service_tier`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_evidence.rs` - `validate_measured_replay`: include pricing identity in duplicate digest and overlap checks so different tiers can replay the same transcript slice while exact duplicate evidence still fails; acceptance proof is `cost_ingest_appends_distinct_receipts_per_service_tier`.

### Phase 2: Tier model precedence and reasoning policy
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_ingest.rs` - `pricing_models_for_tier`: resolve named tier models before top-level default models; acceptance proof is `cost_pricing_prefers_named_default_tier_models`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_pricing.rs` - `pricing helper`: centralize supported reasoning-token policy and amount replay; acceptance proof is `cost_pricing_rejects_unsupported_reasoning_policy` and `cost_evidence_rejects_unsupported_reasoning_policy_on_replay`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module registration`: register `cost_pricing`; acceptance proof is full cargo test.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_evidence.rs` - `validate_measured_replay`: use the shared pricing helper for amount replay and unsupported reasoning-policy rejection; acceptance proof is `cost_evidence_rejects_unsupported_reasoning_policy_on_replay`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: declare `rust/task-registry-flow-cli/src/cost_pricing.rs`; acceptance proof is `.codex/scripts/task-registry release-check all --format json`.

### Phase 3: Tool-bound cost coverage
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_coverage.rs` - `cost_matches_agent`: require matching `boundary_tool_use_id` or contribution `tool_use_ids` when the mutating agent has a tool id; acceptance proof is `cost_coverage_rejects_tool_bound_mutation_without_tool_bound_cost` and `cost_coverage_accepts_exact_tool_bound_cost`.

### Phase 4: Tests, docs, release, and handoff
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs` - `review regression tests`: add focused positive and negative tests for all four review gaps; acceptance proof is focused cargo tests.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `nested test module list`: register `cost_capture_review_tests` without adding lines to already-full `tests/mod.rs`; acceptance proof is full cargo test.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source.required`: declare `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs`; acceptance proof is `.codex/scripts/task-registry release-check all --format json`.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `Cost capture and reporting`: document exact supported reasoning policy, tier-aware dedupe, report grouping, and tool-bound coverage; acceptance proof is docs grep plus release readiness.
- [ ] `[MODIFY]` `docs/provider-usage-adapter-contract.md` - `mutation boundary contract`: state that measured mutation cost evidence must bind to exposed tool-use ids; acceptance proof is docs grep plus release readiness.
- [ ] `[VERIFY]` `scripts/test-release-readiness.sh` - `release readiness`: keep release package assertions passing with the new Rust source; acceptance proof is `bash scripts/test-release-readiness.sh all`.

## Per-Gap Success Criteria

### GAP-001 Service tier receipt dedupe ignores pricing identity
- Current failure: `receipt_exists` treats same transcript and target as duplicate even when service tier, rates, or snapshot hash differ.
- Good behavior: Standard and fast ingests for the same transcript/target append two measured receipts and report distinct tier amounts.
- Forbidden behavior: A fast receipt is skipped because a standard receipt with the same transcript digest already exists.
- Files involved: `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/cost_report.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_appends_distinct_receipts_per_service_tier`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_dedupes_only_identical_pricing_identity`.
- Data/schema/provenance: existing wire schema is unchanged; duplicate identity compares existing `CostPricingSnapshot`, `CostPricingRates`, `CostAmount`, and contribution digest.
- Runtime: `cost-report` shows tier-specific entries and does not merge different tiers.

### GAP-002 Default tier lookup prefers stale top-level models
- Current failure: selecting a default tier returns top-level `models` before matching named `[[service_tiers]]`.
- Good behavior: If `service_tier = "standard"` and a `[[service_tiers]] name = "standard"` exists, named tier rates price the receipt.
- Forbidden behavior: top-level legacy rates are used for a named selected tier.
- Files involved: `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_prefers_named_default_tier_models`.
- Negative test: same test includes stale top-level rates that would produce the wrong amount if used.
- Data/schema/provenance: no schema change; precedence only changes runtime selection.
- Runtime: `cost-ingest` uses named-tier rates deterministically.

### GAP-003 Unsupported reasoning-token policies undercount spend
- Current failure: any non-empty `reasoning_token_policy` passes when reasoning tokens are present, while pricing ignores reasoning tokens.
- Good behavior: `reasoning_tokens_not_billed_separately` is the only supported policy for this release; amount replay uses one helper for ingest and evidence validation.
- Forbidden behavior: unsupported policy such as billing reasoning as output passes ingest or cost-evidence replay.
- Files involved: `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/cost_pricing.rs`, `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs`, `REQUIREMENTS.toml`.
- Positive test: existing no-reasoning and supported-policy pricing tests continue to pass.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unsupported_reasoning_policy` and `cost_evidence_rejects_unsupported_reasoning_policy_on_replay`.
- Data/schema/provenance: pricing snapshot policy is validated as semantic input; receipt schema unchanged.
- Runtime: `cost-report` excludes unsupported-policy measured receipts via `cost-evidence-check`.

### GAP-004 Cost coverage accepts unbound tool evidence
- Current failure: empty `usage_contributions[].tool_use_ids` matches any mutating `agent.tool_use_id`.
- Good behavior: if mutation attribution has a tool id, cost evidence must carry the same boundary tool id or contribution tool id.
- Forbidden behavior: same session/turn/model with no tool evidence covers a tool-bound mutation.
- Files involved: `rust/task-registry-flow-cli/src/cost_coverage.rs`, `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs`, `docs/provider-usage-adapter-contract.md`, `docs/runtime-schemas.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_exact_tool_bound_cost`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_tool_bound_mutation_without_tool_bound_cost`.
- Data/schema/provenance: existing `boundary_tool_use_id` and `usage_contributions[].tool_use_ids` fields are required for tool-bound coverage.
- Runtime: `cost-coverage-check` fails closed until tool-bound evidence is recorded or explicit unmeasured tool-bound evidence exists.

## Validation Plan

Focused:
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_appends_distinct_receipts_per_service_tier`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_dedupes_only_identical_pricing_identity`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_prefers_named_default_tier_models`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unsupported_reasoning_policy`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_unsupported_reasoning_policy_on_replay`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_tool_bound_mutation_without_tool_bound_cost`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_exact_tool_bound_cost`
- `.codex/scripts/task-registry cost-evidence-check --format json`
- `.codex/scripts/task-registry cost-report --format json`
- `.codex/scripts/task-registry cost-coverage-check --format json`
- `.codex/scripts/task-registry source-limit check`

Full:
- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `.codex/scripts/task-registry release-check all --format json`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

`tests/mod.rs` is already exactly 1600 lines, so review regressions are registered as a nested module from `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` and implemented in `rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs`. The expected line-budget impact keeps every touched source, doc, config, and script under 1600 lines. Validate with `.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- `PLAN_ACTIVATE docs/plans/token-capture-review-gap-closure-2026-06-02.md ok`.
- Focused behavior tests pass for all seven named regression tests.
- `.codex/scripts/task-registry cost-evidence-check --format json`, `cost-report --format json`, and `cost-coverage-check --format json` produce expected pass/fail posture for repository evidence.
- `.codex/scripts/task-registry source-limit check` passes.
- `.codex/scripts/task-registry validate`, `verify-chain --format json`, `release-check all --format json`, full cargo test, and `bash scripts/test-release-readiness.sh all` pass.
- `.codex/scripts/task-registry report PLAN-2026-06-02-token-capture-review-gap-closure` reports completed tasks only.
- `.codex/scripts/task-registry metrics` shows no malformed events or chain breaks.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-token-capture-review-gap-closure"

[[behaviors]]
behavior_id = "B-TCR-001-tier-dedupe-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Different service tiers produce distinct measured receipts"
given = "A transcript, target, and tiered pricing snapshot with standard and fast rates"
when = "cost-ingest appends standard then fast receipts for the same transcript and target"
then = "both receipts are appended and cost-report exposes distinct tier-specific measured entries"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_appends_distinct_receipts_per_service_tier"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_appends_distinct_receipts_per_service_tier"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-002-tier-dedupe-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Only identical pricing identity dedupes measured receipts"
given = "A measured receipt already exists for a tier, snapshot hash, rates, amount, target, and transcript digest"
when = "the exact same receipt is appended again, then a different pricing identity is appended"
then = "the identical receipt is skipped and the different pricing identity is appended"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_dedupes_only_identical_pricing_identity"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_dedupes_only_identical_pricing_identity"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-003-tier-precedence-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Named tier rates override stale top-level models"
given = "A snapshot with service_tier standard, stale top-level models, and named service_tiers.standard rates"
when = "cost-ingest selects the standard tier"
then = "the measured amount is calculated from the named tier rates"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_prefers_named_default_tier_models"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_prefers_named_default_tier_models"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-004-tier-precedence-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Stale top-level rates are not used for a named tier"
given = "A snapshot where top-level models would produce an obviously different amount"
when = "cost-ingest selects a named tier"
then = "the stale top-level amount is not returned"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_prefers_named_default_tier_models"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_prefers_named_default_tier_models"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-005-reasoning-policy-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Supported reasoning-token policy prices deterministically"
given = "A transcript with reasoning tokens and a pricing snapshot declaring reasoning_tokens_not_billed_separately"
when = "cost-ingest calculates measured usage"
then = "the receipt records reasoning tokens for provenance and prices input, cached input, and output only"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unsupported_reasoning_policy"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unsupported_reasoning_policy"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-006-reasoning-policy-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Unsupported reasoning-token policy fails closed"
given = "A transcript or measured receipt with reasoning tokens and an unsupported pricing policy"
when = "cost-ingest or cost-evidence replay validates it"
then = "the command fails rather than undercounting reasoning spend"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_unsupported_reasoning_policy_on_replay"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_unsupported_reasoning_policy_on_replay"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-007-tool-coverage-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Tool-bound mutation is covered by exact tool-bound cost evidence"
given = "A mutation receipt with a measured agent tool_use_id"
when = "matching measured or unmeasured cost evidence names the same tool boundary"
then = "cost-coverage-check passes for that mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_exact_tool_bound_cost"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_exact_tool_bound_cost"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TCR-008-tool-coverage-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Tool-bound mutation is not covered by unbound cost evidence"
given = "A mutation receipt with a measured agent tool_use_id"
when = "cost evidence matches session, turn, and model but has no matching tool id"
then = "cost-coverage-check fails for that mutation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_tool_bound_mutation_without_tool_bound_cost"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_tool_bound_mutation_without_tool_bound_cost"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-review-001"
title = "Close token-capture runtime correctness gaps"
status = "active"
kind = "implementation"
reason = "Pricing identity, tier precedence, reasoning-token policy, and tool-bound coverage must be correct before token capture can govern repo mutations."
acceptance_proof = "Behaviors B-TCR-001 through B-TCR-008."
behavior_ids = ["B-TCR-001-tier-dedupe-positive", "B-TCR-002-tier-dedupe-negative", "B-TCR-003-tier-precedence-positive", "B-TCR-004-tier-precedence-negative", "B-TCR-005-reasoning-policy-positive", "B-TCR-006-reasoning-policy-negative", "B-TCR-007-tool-coverage-positive", "B-TCR-008-tool-coverage-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_ingest.rs"
object = "receipt_exists, pricing_models_for_tier, and amount calculation"
required_change = "Use tier-aware receipt identity, prefer named tier models, and call the shared supported reasoning-token pricing helper."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_report.rs"
object = "CostReport grouping key"
required_change = "Group measured report entries by service tier as well as target, status, provider, model, and pricing version."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_evidence.rs"
object = "validate_measured_replay"
required_change = "Include pricing identity in duplicate/overlap replay keys and reject unsupported reasoning policies through the shared pricing helper."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_pricing.rs"
object = "pricing helper"
required_change = "Add canonical amount calculation and supported reasoning policy validation."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module list"
required_change = "Register cost_pricing module."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source.required"
required_change = "Declare cost_pricing.rs and cost_capture_review_tests.rs as release sources."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_coverage.rs"
object = "cost_matches_agent"
required_change = "Require matching tool boundary when agent tool_use_id is present."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/cost_capture_review_tests.rs"
object = "review regression tests"
required_change = "Add positive and negative tests for tier dedupe, tier precedence, reasoning policy, and tool-bound coverage."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "nested test module registration"
required_change = "Register cost_capture_review_tests without modifying tests/mod.rs."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-review-002"
title = "Update documentation and release validation"
status = "active"
kind = "implementation"
reason = "Public contracts must state tier, reasoning-policy, and tool-bound coverage semantics."
acceptance_proof = "Full validation plan plus release readiness."
behavior_ids = ["B-TCR-001-tier-dedupe-positive", "B-TCR-006-reasoning-policy-negative", "B-TCR-008-tool-coverage-negative"]

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "Cost capture and reporting"
required_change = "Document tier-aware dedupe/reporting, supported reasoning policy, and tool-bound coverage."

[[tasks.targets]]
file = "docs/provider-usage-adapter-contract.md"
object = "mutation boundary contract"
required_change = "Require exposed tool-use ids for measured mutation cost coverage."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-review-003"
title = "Superseded overlapping tool coverage task"
status = "cancelled"
kind = "implementation"
reason = "Superseded by TASK-2026-06-02-token-capture-review-001 to keep each changed file mapped to one active task target."
acceptance_proof = "No implementation closure claimed by this cancelled task."
behavior_ids = ["B-TCR-007-tool-coverage-positive", "B-TCR-008-tool-coverage-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_coverage.rs"
object = "superseded target"
required_change = "Cancelled duplicate target; active coverage closure is owned by TASK-2026-06-02-token-capture-review-001."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-review-004"
title = "Superseded overlapping documentation task"
status = "cancelled"
kind = "implementation"
reason = "Superseded by TASK-2026-06-02-token-capture-review-002 to keep each changed file mapped to one active task target."
acceptance_proof = "No implementation closure claimed by this cancelled task."
behavior_ids = ["B-TCR-001-tier-dedupe-positive", "B-TCR-006-reasoning-policy-negative", "B-TCR-008-tool-coverage-negative"]

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "superseded target"
required_change = "Cancelled duplicate target; active documentation closure is owned by TASK-2026-06-02-token-capture-review-002."

```
