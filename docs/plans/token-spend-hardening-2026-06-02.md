# Token Spend Hardening Gap Closure Contract

## Approved Scope

Harden token spend from a measured Codex transcript estimate into governed,
replayable engineering-policy evidence for agent-assisted repositories.

In scope:

- Require deterministic Codex transcript selection, session identity, model
  context, token event digesting, transcript hashing, and pricing snapshot
  hashing for measured cost evidence.
- Replace commit-only cost ingestion with canonical target kind plus target id.
- Validate measured receipts by recomputing usage cost and rejecting overlap.
- Migrate current cost evidence to the canonical receipt shape or correct it to
  unmeasured evidence when provenance cannot be reconstructed.
- Add cost reporting that separates measured, unmeasured, invalid, and
  incomplete evidence.
- Document Codex as the first measured provider adapter and define the future
  provider adapter contract without guessing non-Codex pricing.

Out of scope:

- Remote billing API calls or network telemetry.
- USD conversion for Codex credits.
- Measured DeepSeek, Gemma, or other non-Codex provider support without an
  adapter and pricing source.
- Release publishing, tagging, or automatic push behavior.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/token-spend-hardening-2026-06-02.md` - `closure_contract`: record approved scope, exact targets, success criteria, validation, and schema v2 task manifest.
- [ ] `[VERIFY]` `docs/plans/token-spend-hardening-2026-06-02.md` - `PLAN_ACTIVATE`: `.codex/scripts/task-registry activate docs/plans/token-spend-hardening-2026-06-02.md`.

### Phase 1: Canonical schema and deterministic ingestion

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `CostEvidence`, `CostPricingSnapshot`, `UsageContribution`: add target, session, transcript digest, selected event digest, model context line, pricing snapshot path/hash, service tier, and unmeasured reason code fields.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_ingest.rs` - `codex_transcript_ingest`: remove unsafe latest selection, require explicit transcript path plus session id, require bounded ranges, validate model context, calculate selected event digest, and use target kind/id.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `usage`: document the canonical `cost-ingest codex-transcript` interface.

### Phase 2: Pricing, validation, and overlap protection

- [ ] `[MODIFY]` `docs/pricing/openai-codex-rate-card-2026-06-02.toml` - `pricing_snapshot`: add required schema, effective date, service tier, and stricter provenance fields.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_evidence.rs` - `measured_validation`: recompute cost from usage plus pricing rates, validate transcript and pricing hashes, reject duplicate selected event digests, and reject overlapping measured ranges.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/metrics.rs` - `cost_metrics`: include measured amount totals and unmeasured counts without treating unmeasured as zero.

### Phase 3: Reporting and migration

- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_report.rs` - `cost_report`: add `cost-report [--format json]` grouped by target, provider, model, pricing version, service tier, and status.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module_list`: include cost report.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: route `cost-report`.
- [ ] `[MODIFY]` `docs/task-registry/events.jsonl` - `cost_evidence`: migrate reconstructable cost evidence to canonical measured evidence or append a correcting unmeasured receipt; repair the local receipt chain with canonical tooling.

### Phase 4: Tests and docs

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `cost_capture_tests`: add positive and negative tests for deterministic selection, pricing validation, evidence validation, migration, provider honesty, and reporting.
- [ ] `[NEW]` `docs/provider-usage-adapter-contract.md` - `provider_adapter_contract`: define provider-neutral measured/unmeasured requirements and state Codex is the first measured adapter.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `token_spend_schema`: document canonical cost evidence, ingestion, evidence checking, and reporting.
- [ ] `[MODIFY]` `README.md` - `policy_compliance_cost_evidence`: state token spend is governed evidence, not guaranteed billing truth.
- [ ] `[MODIFY]` `ROADMAP.md` - `token_spend_direction`: align roadmap with provider adapter and cost-per-commit goals.
- [ ] `[MODIFY]` `docs/engineering-policy-compliance.md` - `token_spend_policy`: describe token spend as an engineering-policy compliance artifact.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-007`: update closed gaps and remaining provider limitations.
- [ ] `[MODIFY]` `docs/version-roadmap.toml` - `v2_1_0`: include this plan.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source`: include new provider adapter doc and cost report source.
- [ ] `[MODIFY]` `package.nix` - `release_source`: keep new release-source files installable if the package source list requires explicit paths.

## Per-Gap Success Criteria

### GAP-001: Measured cost evidence is not reproducible enough

- Current failure: Measured receipts record transcript contribution and rates but not transcript hashes, selected event digests, pricing snapshot hashes, session identity, or model context proof.
- Good behavior: Given a bounded Codex transcript range and pricing snapshot, when ingestion runs, then the receipt contains transcript hash, selected event digest, model context line, session id, pricing snapshot path/hash, usage, rates, and calculated amount.
- Forbidden behavior: A measured receipt without replayable transcript and pricing provenance passes validation.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_records_transcript_and_event_digest -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_tampered_transcript_digest -- --nocapture`.
- Data/schema/provenance: Measured receipts must include transcript and pricing hashes.
- Runtime: Local file reads only; no remote telemetry.

### GAP-002: Latest transcript selection can measure the wrong session

- Current failure: `--latest` chooses by modified time and can select stale or unrelated transcripts.
- Good behavior: Given explicit transcript path, session id, target kind/id, and bounded line range, ingestion accepts only matching session evidence.
- Forbidden behavior: `--latest`, missing session id, unbounded range, or session mismatch produces measured evidence.
- Files involved: `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_accepts_explicit_session_and_transcript -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_latest_selector -- --nocapture`.
- Data/schema/provenance: Receipt session id must match transcript `session_meta`.
- Runtime: Ingestion fails closed before cost calculation on ambiguous source selection.

### GAP-003: Target attribution is commit-only and too narrow

- Current failure: Cost ingestion requires `--commit`, while product direction needs cost evidence for commits, plans, tasks, verifiers, landing attempts, retries, release cycles, and sessions.
- Good behavior: Given `--target-kind commit --target-id HEAD`, ingestion resolves and records a commit SHA; given other supported target kinds, ingestion records the exact target id.
- Forbidden behavior: Unknown target kind, missing target id, invalid commit, or legacy `--commit` interface creates measured evidence.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_links_measured_usage_to_commit_target -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_legacy_commit_flag -- --nocapture`.
- Data/schema/provenance: Target kind and target id are first-class receipt fields.
- Runtime: Commit targets resolve through local git.

### GAP-004: Pricing snapshot validation is too weak

- Current failure: Pricing validation does not require effective date, service tier, schema version strictness, duplicate detection, or positive rates.
- Good behavior: Given a valid OpenAI Codex snapshot, ingestion records pricing provenance and hash; malformed snapshots fail before measured evidence is emitted.
- Forbidden behavior: Duplicate models, zero rates, missing service tier, missing effective date, or unpriced model produces measured evidence.
- Files involved: `docs/pricing/openai-codex-rate-card-2026-06-02.toml`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot_accepts_valid_codex_rate_card -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot_rejects_zero_rates -- --nocapture`.
- Data/schema/provenance: Snapshot includes source URL, retrieved timestamp, effective date, service tier, and rates.
- Runtime: Local pricing snapshot only.

### GAP-005: Evidence checker does not recompute or detect overlap

- Current failure: `cost-evidence-check` validates presence of fields but does not recompute amount, verify hashes, or reject double counting.
- Good behavior: Given canonical receipts, evidence checking recomputes amount, validates hashes, and rejects overlapping measured ranges or duplicate event digests.
- Forbidden behavior: Forged amount, changed transcript, changed pricing file, duplicate digest, or overlapping range passes.
- Files involved: `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/metrics.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_accepts_non_overlapping_ranges -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_duplicate_selected_event_digest -- --nocapture`.
- Data/schema/provenance: Selected token event digests are unique unless marked unmeasured.
- Runtime: Validation reads only local receipt, transcript, and pricing files.

### GAP-006: Current cost evidence must be migrated or corrected

- Current failure: The existing cost receipt lacks canonical replay fields.
- Good behavior: If existing evidence can be reconstructed from transcript and pricing files, it is migrated to canonical measured evidence and the receipt chain is repaired; otherwise a correction records unmeasured evidence.
- Forbidden behavior: Old measured receipt shape remains accepted after migration.
- Files involved: `docs/task-registry/events.jsonl`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_migration_preserves_receipt_chain_integrity -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_legacy_measured_receipt_after_migration -- --nocapture`.
- Data/schema/provenance: Legacy measured cost evidence is no longer canonical measured evidence.
- Runtime: Chain repair uses `.codex/scripts/task-registry verify-chain --repair`.

### GAP-007: Provider scope can be overstated

- Current failure: Token spend language can imply universal provider cost capture.
- Good behavior: Docs and runtime state Codex is the first measured adapter; unknown providers produce unmeasured evidence unless an adapter contract and pricing source exist.
- Forbidden behavior: DeepSeek, Gemma, or unknown model names become measured from name inference.
- Files involved: `docs/provider-usage-adapter-contract.md`, `docs/runtime-schemas.md`, `README.md`, `docs/engineering-policy-compliance.md`, `rust/task-registry-flow-cli/src/cost_ingest.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml provider_adapter_records_unknown_model_as_unmeasured -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml provider_adapter_rejects_model_name_pricing_inference -- --nocapture`.
- Data/schema/provenance: Non-Codex measured evidence requires explicit adapter and pricing.
- Runtime: Unknown pricing fails closed.

### GAP-008: Reporting can collapse unknown into zero

- Current failure: Metrics count cost evidence statuses but do not provide target-level cost reporting or distinguish incomplete cost-per-commit evidence.
- Good behavior: `cost-report` groups measured and unmeasured evidence separately and emits cost per commit only from complete, non-overlapping measured receipts.
- Forbidden behavior: Missing evidence is reported as zero cost.
- Files involved: `rust/task-registry-flow-cli/src/cost_report.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/metrics.rs`, `rust/task-registry-flow-cli/src/reviewer_report.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_emits_measured_cost_per_commit -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_does_not_emit_zero_for_unmeasured_commit -- --nocapture`.
- Data/schema/provenance: Reports preserve status, provider, model, pricing version, service tier, and target identity.
- Runtime: Report reads local receipts only.

## Validation Plan

Focused:

- `.codex/scripts/task-registry source-limit check`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report -- --nocapture`
- `.codex/scripts/task-registry cost-evidence-check --format json`
- `.codex/scripts/task-registry cost-report --format json`
- `.codex/scripts/task-registry verify-chain --format json`

Full:

- `.codex/scripts/task-registry validate`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml --all-targets -- -D warnings`
- `bash scripts/test-release-readiness.sh all`
- `.codex/scripts/task-registry report PLAN-2026-06-02-token-spend-hardening`
- `.codex/scripts/task-registry metrics`
- `.codex/scripts/task-registry archive-completed`

## Source File Limit

`rust/task-registry-flow-cli/src/tests/mod.rs` is already at 1600 lines. This
plan avoids adding a new test module and places focused token-spend tests in
the already included `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
All implementation must pass `.codex/scripts/task-registry source-limit check`.

## Walkthrough Evidence

- `PLAN_ACTIVATE` output for this plan.
- Focused positive and negative cargo test output.
- Cost evidence check JSON output with zero failures.
- Cost report JSON output showing measured and unmeasured evidence separately.
- Receipt-chain verification after any ledger migration or correction.
- Source-limit, registry validation, full cargo test, clippy, and release-readiness output.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-token-spend-hardening"

[[behaviors]]
behavior_id = "B-001-reproducible-cost-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Measured cost receipt records replayable transcript and pricing digests"
given = "A bounded Codex transcript range and a valid pricing snapshot"
when = "cost ingestion runs"
then = "the measured receipt includes transcript hash, selected event digest, pricing hash, session id, model context line, usage, rates, and amount"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_records_transcript_and_event_digest -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_records_transcript_and_event_digest -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-002-reproducible-cost-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Tampered transcript digest fails measured validation"
given = "A measured cost receipt whose transcript hash no longer matches its transcript"
when = "cost evidence validation runs"
then = "the receipt fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_tampered_transcript_digest -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_tampered_transcript_digest -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-003-deterministic-selection-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Explicit transcript and session selection succeeds"
given = "A transcript with matching session_meta and bounded token_count events"
when = "cost ingestion uses --transcript-path, --session-id, --since-line, and --until-line"
then = "the measured report targets exactly that source range"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_accepts_explicit_session_and_transcript -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_accepts_explicit_session_and_transcript -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-004-deterministic-selection-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Latest transcript selector is rejected"
given = "A cost ingestion request using --latest"
when = "the request is parsed"
then = "the runtime rejects it before source selection"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_latest_selector -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_latest_selector -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-005-target-attribution-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Measured cost links to canonical commit target"
given = "A valid commit target, Codex transcript range, and pricing snapshot"
when = "cost ingestion runs with --target-kind commit --target-id HEAD"
then = "the receipt stores the resolved commit sha as the canonical target"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_links_measured_usage_to_commit_target -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_links_measured_usage_to_commit_target -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-006-target-attribution-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Legacy commit flag is rejected"
given = "A cost ingestion request using --commit"
when = "the request is parsed"
then = "the runtime rejects the legacy interface"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_legacy_commit_flag -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_legacy_commit_flag -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-007-pricing-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Valid Codex pricing snapshot is accepted"
given = "A pricing snapshot with schema version, effective date, service tier, and positive rates"
when = "cost ingestion loads the snapshot"
then = "the measured receipt records the pricing snapshot hash and service tier"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot_accepts_valid_codex_rate_card -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot_accepts_valid_codex_rate_card -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-008-pricing-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Zero pricing rates are rejected"
given = "A pricing snapshot with zero rates"
when = "cost ingestion loads the snapshot"
then = "the runtime rejects it before measured evidence is emitted"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot_rejects_zero_rates -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml pricing_snapshot_rejects_zero_rates -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-009-evidence-positive"
gap_id = "GAP-005"
polarity = "positive"
title = "Non-overlapping measured ranges validate"
given = "Canonical measured receipts with distinct selected event digests"
when = "cost evidence validation runs"
then = "the report has no failures"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_accepts_non_overlapping_ranges -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_accepts_non_overlapping_ranges -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-010-evidence-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Duplicate selected event digest fails"
given = "Two measured receipts with the same selected event digest"
when = "cost evidence validation runs"
then = "validation fails"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_duplicate_selected_event_digest -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_duplicate_selected_event_digest -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-011-migration-positive"
gap_id = "GAP-006"
polarity = "positive"
title = "Receipt chain remains valid after cost evidence migration"
given = "Current ledger cost evidence is migrated or corrected"
when = "receipt-chain verification runs"
then = "the chain validates"
confirmation = ".codex/scripts/task-registry verify-chain --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry verify-chain --format json"
expected_exit = 0

[[behaviors]]
behavior_id = "B-012-migration-negative"
gap_id = "GAP-006"
polarity = "negative"
title = "Legacy measured receipt shape is rejected"
given = "A measured cost receipt missing canonical digest fields"
when = "cost evidence validation runs"
then = "the receipt fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_legacy_measured_receipt_after_migration -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_evidence_rejects_legacy_measured_receipt_after_migration -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-013-provider-positive"
gap_id = "GAP-007"
polarity = "positive"
title = "Unknown provider usage is explicit unmeasured evidence"
given = "A provider or model without a supported adapter or pricing source"
when = "cost posture is recorded"
then = "the evidence is unmeasured with a reason and no amount"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml provider_adapter_records_unknown_model_as_unmeasured -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml provider_adapter_records_unknown_model_as_unmeasured -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-014-provider-negative"
gap_id = "GAP-007"
polarity = "negative"
title = "Model name pricing inference is rejected"
given = "A non-Codex model name with no provider adapter"
when = "measured cost evidence is attempted"
then = "the runtime rejects measured cost creation"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml provider_adapter_rejects_model_name_pricing_inference -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml provider_adapter_rejects_model_name_pricing_inference -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-015-report-positive"
gap_id = "GAP-008"
polarity = "positive"
title = "Cost report emits measured cost per commit"
given = "Canonical measured commit evidence exists"
when = "cost-report runs"
then = "the report includes target-level measured cost"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_emits_measured_cost_per_commit -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_emits_measured_cost_per_commit -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-016-report-negative"
gap_id = "GAP-008"
polarity = "negative"
title = "Unmeasured commit is not reported as zero cost"
given = "Only unmeasured commit evidence exists"
when = "cost-report runs"
then = "the report lists unmeasured evidence without a zero measured amount"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_does_not_emit_zero_for_unmeasured_commit -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_does_not_emit_zero_for_unmeasured_commit -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-017-token-spend-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Token spend hardening validation passes"
given = "Implementation and docs are complete"
when = "focused validation runs"
then = "source limit, evidence check, report, receipt chain, and focused tests pass"
confirmation = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry cost-evidence-check --format json && .codex/scripts/task-registry cost-report --format json && .codex/scripts/task-registry verify-chain --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry cost-evidence-check --format json && .codex/scripts/task-registry cost-report --format json && .codex/scripts/task-registry verify-chain --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-token-spend-hardening-001"
behavior_ids = [
  "B-001-reproducible-cost-positive",
  "B-002-reproducible-cost-negative",
  "B-003-deterministic-selection-positive",
  "B-004-deterministic-selection-negative",
  "B-005-target-attribution-positive",
  "B-006-target-attribution-negative",
]
status = "planned"
title = "Implement canonical deterministic cost ingestion"
kind = "implementation"
reason = "Token spend evidence must be deterministic and target-attributed before it can support engineering policy compliance."
acceptance_proof = "Behaviors B-001 through B-006."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "CostEvidence and UsageContribution"
required_change = "Add canonical digest, session, pricing hash, service tier, and target evidence fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_ingest.rs"
object = "codex_transcript_ingest"
required_change = "Require explicit session and bounded transcript range, reject legacy selectors, compute evidence digests, and record target kind/id."

[[tasks]]
task_id = "TASK-2026-06-02-token-spend-hardening-002"
behavior_ids = [
  "B-007-pricing-positive",
  "B-008-pricing-negative",
  "B-009-evidence-positive",
  "B-010-evidence-negative",
  "B-011-migration-positive",
  "B-012-migration-negative",
]
status = "planned"
title = "Harden pricing and cost evidence validation"
kind = "schema"
reason = "Measured cost evidence must be replayable, priced from governed snapshots, and protected against double counting."
acceptance_proof = "Behaviors B-007 through B-012."

[[tasks.targets]]
file = "docs/pricing/openai-codex-rate-card-2026-06-02.toml"
object = "pricing_snapshot"
required_change = "Add strict pricing provenance and service-tier fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_evidence.rs"
object = "measured_validation"
required_change = "Recompute amount, validate file hashes, reject legacy measured receipts, and reject duplicate event digests."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "cost_metrics"
required_change = "Include measured amount totals and unmeasured counts honestly."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "MetricsReport"
required_change = "Add measured cost amount totals to metrics schema."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "cost_evidence"
required_change = "Migrate or correct current cost receipt and repair receipt chain."

[[tasks]]
task_id = "TASK-2026-06-02-token-spend-hardening-003"
behavior_ids = [
  "B-013-provider-positive",
  "B-014-provider-negative",
  "B-015-report-positive",
  "B-016-report-negative",
]
status = "planned"
title = "Add provider honesty and cost reporting"
kind = "implementation"
reason = "Reports must distinguish measured spend from unmeasured provider gaps and must not collapse unknown cost into zero."
acceptance_proof = "Behaviors B-013 through B-016."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_report.rs"
object = "cost_report"
required_change = "Add target-level measured and unmeasured token spend reporting."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module_list"
required_change = "Include cost_report module."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Route cost-report."

[[tasks.targets]]
file = "docs/provider-usage-adapter-contract.md"
object = "provider_adapter_contract"
required_change = "Document provider-neutral measured/unmeasured requirements."

[[tasks]]
task_id = "TASK-2026-06-02-token-spend-hardening-004"
behavior_ids = [
  "B-001-reproducible-cost-positive",
  "B-002-reproducible-cost-negative",
  "B-003-deterministic-selection-positive",
  "B-004-deterministic-selection-negative",
  "B-005-target-attribution-positive",
  "B-006-target-attribution-negative",
  "B-007-pricing-positive",
  "B-008-pricing-negative",
  "B-009-evidence-positive",
  "B-010-evidence-negative",
  "B-012-migration-negative",
  "B-013-provider-positive",
  "B-014-provider-negative",
  "B-015-report-positive",
  "B-016-report-negative",
  "B-017-token-spend-validation",
]
status = "planned"
title = "Add tests, docs, release-source alignment, and final validation"
kind = "validation"
reason = "The migration must have positive and negative behavior coverage and public claims must match the runtime."
acceptance_proof = "Behaviors B-001 through B-017."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "cost_capture_tests"
required_change = "Add comprehensive positive and negative token spend tests without extending tests/mod.rs."

[[tasks.targets]]
file = "docs/plans/token-spend-hardening-2026-06-02.md"
object = "closure_contract"
required_change = "Track approved scope, behavior verifiers, target files, and validation evidence."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "token_spend_schema"
required_change = "Document canonical cost evidence and reporting."

[[tasks.targets]]
file = "README.md"
object = "policy_compliance_cost_evidence"
required_change = "State token spend evidence accurately."

[[tasks.targets]]
file = "ROADMAP.md"
object = "token_spend_direction"
required_change = "Align product direction with provider adapter and cost-per-commit goals."

[[tasks.targets]]
file = "docs/engineering-policy-compliance.md"
object = "token_spend_policy"
required_change = "Document token spend as engineering policy compliance evidence."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-007"
required_change = "Update gap status and remaining limitations."

[[tasks.targets]]
file = "docs/version-roadmap.toml"
object = "v2_1_0"
required_change = "Include this hardening plan."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source"
required_change = "Include new source and docs files."

[[tasks.targets]]
file = "package.nix"
object = "release_source"
required_change = "Keep release-source packaging aligned when explicit path lists require it."
```
