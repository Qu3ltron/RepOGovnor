# Provider Usage Ingestion Gap Closure Contract

## Approved Scope

Implement actual Codex provider-usage ingestion, pricing snapshots, and commit
attribution without mocks, synthetic usage, or guessed pricing.

In scope:

- Ingest real local Codex transcript `token_count` events as the usage source.
- Use an OpenAI Codex token-based rate-card snapshot as pricing input.
- Attribute measured usage and computed credit spend to an explicit commit.
- Record contribution evidence: transcript path, line range, event count, model,
  token counts, pricing snapshot path, and commit SHA.
- Validate and document that Codex transcript format is not stable, so ingestion
  fails closed on missing or malformed actual usage evidence.

Out of scope:

- Remote telemetry.
- Synthetic usage fixtures as proof of completion.
- USD conversion for ChatGPT Codex usage. This slice records Codex credits,
  because OpenAI's Codex rate card prices Codex usage in credits.
- Non-Codex providers.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/provider-usage-ingestion-2026-06-02.md` - `closure_contract`: create and activate this contract before implementation.
- [ ] `[VERIFY]` `docs/plans/provider-usage-ingestion-2026-06-02.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/provider-usage-ingestion-2026-06-02.md`.

### Phase 1: Runtime ingestion

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `cost_evidence_contributions`: add typed transcript contribution evidence and pricing-rate fields.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_ingest.rs` - `codex_transcript_ingest`: add `cost-ingest codex-transcript` using actual transcript token-count events.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module_list`: include cost ingestion module.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: route `cost-ingest`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_evidence.rs` - `measured_validation`: require contribution and pricing evidence for measured cost receipts.

### Phase 2: Pricing and proof

- [ ] `[NEW]` `docs/pricing/openai-codex-rate-card-2026-06-02.toml` - `official_rate_card_snapshot`: record OpenAI Codex token-based credit rates with source URL and retrieval date.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `cost_ingest_tests`: prove parser/calculator rejects missing usage and computes only from actual-shaped transcript token-count records.
- [ ] `[MODIFY]` `README.md` - `cost_ingest`: document live Codex transcript ingestion and credit-based commit attribution.
- [ ] `[MODIFY]` `ROADMAP.md` - `cost_evidence`: move GP-007 posture from schema-only to live Codex transcript ingestion.
- [ ] `[MODIFY]` `docs/engineering-policy-compliance.md` - `token_cost_evidence`: document measured Codex transcript usage and credit attribution.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `cost_ingest_schema`: document command, transcript contribution evidence, pricing snapshot, and fail-closed behavior.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-007`: record live Codex ingestion evidence and remaining non-Codex/provider gaps.
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `v2_1_0`: include this plan.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source`: include new source and pricing snapshot files.

## Per-Gap Success Criteria

### GAP-001: Actual provider usage is not ingested

- Current failure: Cost evidence can validate receipts, but no command ingests real usage from a provider-backed source.
- Good behavior: Given an actual Codex transcript containing `token_count` events, when `cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --format json` runs, then the report contains measured usage derived from real transcript token counts.
- Forbidden behavior: Missing transcript, missing token-count events, malformed token counts, or unknown model pricing must fail instead of producing estimated or synthetic spend.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `.codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --format json`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_usage -- --nocapture`
- Data/schema/provenance: Usage source must be a real transcript path and line range, not test narration or a generated receipt.
- Runtime: Local transcript read only; no network telemetry.

### GAP-002: Pricing snapshots are not first-class evidence

- Current failure: Measured cost can require pricing, but the repo has no declared pricing snapshot source.
- Good behavior: Given the OpenAI Codex rate card snapshot, when ingestion runs for a known Codex model, then the amount uses input, cached input, and output credit rates from the snapshot.
- Forbidden behavior: Unknown models, missing snapshot metadata, missing rates, or guessed USD conversion must fail.
- Files involved: `docs/pricing/openai-codex-rate-card-2026-06-02.toml`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_computes_credit_amount_from_pricing_snapshot -- --nocapture`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_unknown_pricing_model -- --nocapture`
- Data/schema/provenance: Snapshot declares `source_url = "https://help.openai.com/en/articles/20001106-codex-rate-card"` and rates per 1M tokens.
- Runtime: Uses local snapshot; live source was checked before writing this plan.

### GAP-003: Commit attribution is not recorded

- Current failure: Cost evidence can target commits, but no command creates commit-linked measured usage receipts.
- Good behavior: Given an explicit commit SHA and actual transcript usage, when ingestion runs with `--append-receipt`, then a measured cost receipt targets the commit and records the transcript contribution lines.
- Forbidden behavior: Commit attribution without an explicit commit, an invalid commit, or missing usage contribution evidence must fail.
- Files involved: `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `docs/runtime-schemas.md`.
- Positive test: `.codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --append-receipt --format json`
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture`
- Data/schema/provenance: Receipt target kind is `commit`; id is a resolved local commit SHA.
- Runtime: Appends local receipt only when `--append-receipt` is present.

## Validation Plan

Focused:

- `.codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --format json`
- `.codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --append-receipt --format json`
- `.codex/scripts/task-registry cost-evidence-check --format json`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest -- --nocapture`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml --all-targets -- -D warnings`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

Expected impact is under 1600 lines. New runtime code goes in
`cost_ingest.rs`; `tests/mod.rs` is already at the limit, so tests are added to
the existing included `model_attribution_tests.rs` module.

## Walkthrough Evidence

- Official OpenAI Codex rate card source check.
- Live transcript ingestion JSON output from an actual Codex transcript.
- Appended measured cost receipt verified by `cost-evidence-check`.
- Landing verification, registry report, metrics, source-limit, validation,
  receipt-chain, full tests, clippy, and release readiness.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-provider-usage-ingestion"

[[behaviors]]
behavior_id = "B-001-live-codex-usage-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Actual Codex transcript usage is ingested"
given = "The local machine has a Codex transcript with token_count events"
when = "cost-ingest codex-transcript runs with an actual transcript path, line range, and the official pricing snapshot"
then = "the JSON report contains measured usage from real transcript token-count events"
confirmation = ".codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-live-codex-usage-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Missing usage evidence fails closed"
given = "A transcript source has no token_count usage events"
when = "cost ingestion is attempted"
then = "the runtime rejects ingestion instead of inventing usage"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_usage -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_usage -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-pricing-snapshot-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Pricing snapshot rates calculate credit amount"
given = "The OpenAI Codex pricing snapshot contains a model rate"
when = "usage is ingested for that model"
then = "credit amount is calculated from input, cached input, and output token rates"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_computes_credit_amount_from_pricing_snapshot -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_computes_credit_amount_from_pricing_snapshot -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-pricing-snapshot-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Unknown pricing model fails closed"
given = "Usage names a model missing from the pricing snapshot"
when = "cost ingestion runs"
then = "the runtime fails instead of guessing a rate"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_unknown_pricing_model -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_unknown_pricing_model -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-commit-attribution-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Measured usage receipt is commit attributed"
given = "Actual Codex usage and pricing snapshot evidence exist"
when = "cost-ingest runs with --append-receipt and --target-kind commit --target-id HEAD"
then = "a measured cost receipt targets the resolved commit and records contribution evidence"
confirmation = ".codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --append-receipt --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry cost-ingest codex-transcript --transcript-path <local-private-codex-transcript> --since-line 14128 --until-line 14642 --pricing-snapshot docs/pricing/openai-codex-rate-card-2026-06-02.toml --target-kind commit --target-id HEAD --append-receipt --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-commit-attribution-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Missing commit attribution fails closed"
given = "Actual usage exists but no commit target is supplied"
when = "cost ingestion is attempted"
then = "the runtime rejects the request"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_ingest_rejects_missing_commit -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007-provider-ingestion-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Provider usage ingestion validation passes"
given = "Runtime ingestion and docs are updated"
when = "focused validation runs"
then = "source limit and cost evidence checks pass"
confirmation = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry cost-evidence-check --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry cost-evidence-check --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-provider-usage-ingestion-001"
behavior_ids = [
  "B-001-live-codex-usage-positive",
  "B-002-live-codex-usage-negative",
  "B-003-pricing-snapshot-positive",
  "B-004-pricing-snapshot-negative",
  "B-006-commit-attribution-negative",
]
status = "planned"
title = "Implement actual Codex transcript cost ingestion"
kind = "implementation"
reason = "GP-007 requires real provider usage ingestion and pricing evidence before honest spend metrics."
acceptance_proof = "Behaviors B-001, B-002, B-003, B-004, and B-006."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "cost_evidence_contributions"
required_change = "Add transcript contribution and pricing-rate evidence fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_ingest.rs"
object = "codex_transcript_ingest"
required_change = "Parse real Codex token_count events, calculate credit cost, and optionally append receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module_list"
required_change = "Include cost_ingest module."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Route cost-ingest command and usage."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_evidence.rs"
object = "measured_validation"
required_change = "Require contribution and pricing evidence for measured receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "cost_ingest_tests"
required_change = "Cover parser and fail-closed behavior."

[[tasks]]
task_id = "TASK-2026-06-02-provider-usage-ingestion-002"
behavior_ids = [
  "B-005-commit-attribution-positive",
  "B-007-provider-ingestion-validation",
]
status = "planned"
title = "Add pricing snapshot, docs, and live commit attribution proof"
kind = "governance"
reason = "Measured cost evidence must carry official pricing provenance and commit attribution."
acceptance_proof = "Behaviors B-005 and B-007."

[[tasks.targets]]
file = "docs/pricing/openai-codex-rate-card-2026-06-02.toml"
object = "official_rate_card_snapshot"
required_change = "Record official OpenAI Codex token-based credit rates."

[[tasks.targets]]
file = "README.md"
object = "cost_ingest"
required_change = "Document live transcript ingestion and commit attribution."

[[tasks.targets]]
file = "ROADMAP.md"
object = "cost_evidence"
required_change = "Update token/cost evidence posture."

[[tasks.targets]]
file = "docs/engineering-policy-compliance.md"
object = "token_cost_evidence"
required_change = "Document measured Codex transcript usage evidence."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "cost_ingest_schema"
required_change = "Document cost-ingest command and receipt contribution schema."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-007"
required_change = "Update current evidence and remaining gaps."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "v2_1_0"
required_change = "Add this plan to release evidence."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source"
required_change = "Include new ingestion source and pricing snapshot."

[[tasks.targets]]
file = "docs/plans/provider-usage-ingestion-2026-06-02.md"
object = "closure_contract"
required_change = "Track approved scope, bounded transcript verifiers, targets, and validation evidence."

[[tasks.targets]]
file = "docs/task-registry.toml"
object = "active_registry"
required_change = "Record activated provider usage ingestion tasks."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "cost_receipt"
required_change = "Append bounded actual Codex transcript cost evidence receipt and activation receipts."
```
