# Token Capture Hardening Gap Closure Contract

## Approved Scope

Close the remaining token-capture gaps by making Codex cost evidence public-safe,
boundary-aware, target-validated, tier-priced, and coverage-checked. Codex remains
the first measured adapter example. Non-Codex providers, unknown pricing tiers,
unavailable transcripts, and unpriced models must produce explicit `unmeasured`
evidence instead of guessed measured spend.

In scope:

- Remove or migrate private transcript paths from tracked public evidence.
- Add boundary-aware Codex cost capture using hook transcript/session/turn/tool
  evidence.
- Add a canonical unmeasured cost-recording command.
- Validate governed cost targets before measured or unmeasured receipts are
  recorded.
- Require service-tier pricing and explicit reasoning-token semantics.
- Add cost coverage checks that join mutation attribution to measured or
  unmeasured cost evidence.
- Make `cost-report` totals depend on replay-valid measured evidence.
- Update docs, skills, readiness checks, and release gates.

Out of scope:

- Non-Codex measured adapters.
- Remote billing telemetry, hosted analytics, or USD conversion.
- Automatic final release publication or final tag push.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/token-capture-hardening-2026-06-02.md` - `closure_contract`: create and activate this contract before implementation; acceptance proof: `PLAN_ACTIVATE`.
- [ ] `[VERIFY]` `docs/task-registry.toml` - `registry_activation`: `.codex/scripts/task-registry activate docs/plans/token-capture-hardening-2026-06-02.md`.

### Phase 1: Public-safe evidence

- [ ] `[MODIFY]` `docs/task-registry/events.jsonl` - `public_cost_evidence`: remove private measured transcript evidence from tracked public receipts or convert it to explicit public-safe unmeasured evidence; acceptance proof: `B-TOKEN-CAPTURE-001-public-evidence-positive` and `B-TOKEN-CAPTURE-002-public-evidence-negative`.
- [ ] `[MODIFY]` `docs/plans/provider-usage-ingestion-2026-06-02.md` - `historical_cost_examples`: replace private absolute transcript command examples with public-safe placeholders and current canonical target flags; acceptance proof: `B-TOKEN-CAPTURE-002-public-evidence-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `public_cost_path_gate`: reject tracked public measured cost receipts or docs with private Codex transcript paths; acceptance proof: `B-TOKEN-CAPTURE-002-public-evidence-negative`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `public_cost_path_gate`: fail release readiness on private transcript path leakage; acceptance proof: `B-TOKEN-CAPTURE-002-public-evidence-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/release_source_tests.rs` - `private_cost_path_tests`: add positive sanitized and negative private-path checks; acceptance proof: `B-TOKEN-CAPTURE-002-public-evidence-negative`.

### Phase 2: Cost schema, targets, and unmeasured records

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `cost_evidence_contract`: add boundary ids, coverage fields, service-tier pricing semantics, reasoning-token policy, and unmeasured reason codes; acceptance proof: `B-TOKEN-CAPTURE-003-unmeasured-record-positive` through `B-TOKEN-CAPTURE-006-target-validation-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_record.rs` - `unmeasured_cost_record`: add `cost-record unmeasured` for explicit unknown spend; acceptance proof: `B-TOKEN-CAPTURE-003-unmeasured-record-positive` and `B-TOKEN-CAPTURE-004-unmeasured-record-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_targets.rs` - `target_resolver`: validate commit, plan, task, verifier-run, landing-attempt, retry, release-cycle, and session targets; acceptance proof: `B-TOKEN-CAPTURE-005-target-validation-positive` and `B-TOKEN-CAPTURE-006-target-validation-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `module_list`: include new cost modules; acceptance proof: focused cost tests.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/runtime.rs` - `dispatcher`: route `cost-record` and `cost-coverage-check`; acceptance proof: focused cost tests.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_source`: add new runtime and test files; acceptance proof: `.codex/scripts/task-registry release-check all --format json`.

### Phase 3: Boundary-aware Codex capture

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/mutation_hook.rs` - `codex_cost_boundary`: capture Codex transcript path, session id, model, turn id, tool use id, target paths, and line watermark when hooks expose them; acceptance proof: `B-TOKEN-CAPTURE-007-boundary-capture-positive` and `B-TOKEN-CAPTURE-008-boundary-capture-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model_attribution.rs` - `boundary_validation`: validate measured Codex mutation attribution includes boundary evidence when available; acceptance proof: `B-TOKEN-CAPTURE-007-boundary-capture-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_ingest.rs` - `boundary_ingest`: allow ingestion from recorded boundaries and reject manual ranges outside the boundary; acceptance proof: `B-TOKEN-CAPTURE-007-boundary-capture-positive` and `B-TOKEN-CAPTURE-008-boundary-capture-negative`.
- [ ] `[MODIFY]` `.codex/hooks.json` - `codex_hook_projection`: ensure pre/post hook config remains wired for boundary evidence; acceptance proof: release readiness.
- [ ] `[MODIFY]` `hooks/codex-hooks.json` - `codex_hook_projection`: mirror hook projection; acceptance proof: release readiness.
- [ ] `[MODIFY]` `templates/.codex/hooks.json.template` - `codex_hook_projection`: mirror hook template; acceptance proof: release readiness.

### Phase 4: Pricing and reasoning semantics

- [ ] `[MODIFY]` `docs/pricing/openai-codex-rate-card-2026-06-02.toml` - `tiered_pricing_snapshot`: represent standard and fast tier rates with governed provenance; acceptance proof: `B-TOKEN-CAPTURE-009-tier-pricing-positive` and `B-TOKEN-CAPTURE-010-tier-pricing-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_pricing.rs` - `pricing_loader`: validate tier/model uniqueness, calculate amount, and enforce reasoning-token policy; acceptance proof: `B-TOKEN-CAPTURE-009-tier-pricing-positive` through `B-TOKEN-CAPTURE-012-reasoning-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_evidence.rs` - `tiered_replay`: replay measured amounts from tiered pricing semantics and reject ambiguous reasoning-token evidence; acceptance proof: `B-TOKEN-CAPTURE-010-tier-pricing-negative` and `B-TOKEN-CAPTURE-012-reasoning-negative`.

### Phase 5: Coverage and reporting

- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cost_coverage.rs` - `coverage_check`: add `cost-coverage-check [--format json]` joining mutation attribution to cost evidence; acceptance proof: `B-TOKEN-CAPTURE-013-coverage-positive` and `B-TOKEN-CAPTURE-014-coverage-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/cost_report.rs` - `verified_totals`: total only replay-valid measured receipts, split by service tier, and report invalid/uncovered/unmeasured state separately; acceptance proof: `B-TOKEN-CAPTURE-015-report-positive` and `B-TOKEN-CAPTURE-016-report-negative`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/metrics.rs` - `cost_coverage_counts`: count verified, invalid, covered, uncovered, and unmeasured cost posture; acceptance proof: `B-TOKEN-CAPTURE-013-coverage-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - `metrics_report`: add coverage count fields; acceptance proof: `B-TOKEN-CAPTURE-013-coverage-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/reviewer_report.rs` - `cost_posture_summary`: report token-cost coverage without claiming total spend; acceptance proof: focused cost report tests.

### Phase 6: Behavioral tests

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` - `token_capture_hardening_tests`: add comprehensive positive and negative behavior tests without exceeding the source limit; acceptance proof: focused cost tests.
- [ ] `[VERIFY]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `line_budget`: do not add new module declarations because this file is already at 1600 lines; acceptance proof: `.codex/scripts/task-registry source-limit check`.

### Phase 7: Docs, projections, and release readiness

- [ ] `[MODIFY]` `README.md` - `cost_capture_posture`: state governed Codex cost evidence, not automatic total spend; acceptance proof: `B-TOKEN-CAPTURE-017-docs-positive` and `B-TOKEN-CAPTURE-018-docs-negative`.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `cost_capture_schema`: document boundary capture, unmeasured records, tiered pricing, coverage checks, and verified reporting; acceptance proof: `B-TOKEN-CAPTURE-017-docs-positive`.
- [ ] `[MODIFY]` `docs/provider-usage-adapter-contract.md` - `adapter_contract`: require service tier, reasoning-token semantics, and public-safe evidence; acceptance proof: `B-TOKEN-CAPTURE-017-docs-positive`.
- [ ] `[MODIFY]` `docs/engineering-policy-compliance.md` - `cost_policy_evidence`: explain measured/unmeasured cost evidence as engineering-policy evidence; acceptance proof: `B-TOKEN-CAPTURE-017-docs-positive`.
- [ ] `[MODIFY]` `docs/gap-pipeline.md` - `GP-007`: update current evidence and remaining non-Codex gaps; acceptance proof: `B-TOKEN-CAPTURE-017-docs-positive`.
- [ ] `[MODIFY]` `ROADMAP.md` - `token_cost_evidence`: reflect closed hardening and remaining future work; acceptance proof: `B-TOKEN-CAPTURE-017-docs-positive`.
- [ ] `[MODIFY]` `skills/task-registry-flow/SKILL.md` - `cost_commands`: add cost-record and cost-coverage guidance; acceptance proof: release readiness.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow.md` - `cost_commands`: mirror skill guidance; acceptance proof: release readiness.
- [ ] `[MODIFY]` `.agents/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror skill guidance; acceptance proof: release readiness.
- [ ] `[MODIFY]` `.cursor/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror skill guidance; acceptance proof: release readiness.
- [ ] `[MODIFY]` `.claude/skills/task-registry-flow/SKILL.md` - `cost_commands`: mirror skill guidance; acceptance proof: release readiness.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `cost_capture_readiness`: assert commands, pricing assets, public-safe evidence, and no private transcript path leakage; acceptance proof: `bash scripts/test-release-readiness.sh all`.

## Per-Gap Success Criteria

### GAP-001: Public tracked cost evidence contains private local transcript paths

- Current failure: `docs/task-registry/events.jsonl` and historical plan docs include private Codex transcript paths that are not public-replayable.
- Good behavior: Given tracked public files, when release checks run, then no private Codex transcript paths or measured public receipts with private absolute `source_path` values are accepted.
- Forbidden behavior: A tracked measured receipt or public doc containing a private Codex transcript path passes release readiness.
- Files involved: `docs/task-registry/events.jsonl`, `docs/plans/provider-usage-ingestion-2026-06-02.md`, `rust/task-registry-flow-cli/src/release_checks.rs`, `scripts/test-release-readiness.sh`, `rust/task-registry-flow-cli/src/tests/release_source_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence_allows_sanitized_paths -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence_rejects_private_transcript_paths -- --nocapture`.
- Data/schema/provenance: Public measured receipts must use repo-local replay fixtures or be recorded as unmeasured public evidence.
- Runtime: Release checks fail before package publication.

### GAP-002: Cost capture boundaries are caller-selected instead of governed

- Current failure: `cost-ingest` trusts caller-supplied transcript line ranges after replay, but it does not prove that the range maps to the mutating turn.
- Good behavior: Given Codex pre/post hook boundary receipts, when ingestion runs from that boundary, then the transcript range is derived from the recorded boundary and linked to session/turn/tool/target evidence.
- Forbidden behavior: Missing post boundary, mismatched session/turn/tool, reversed range, or a manual range outside the boundary produces measured evidence.
- Files involved: `rust/task-registry-flow-cli/src/mutation_hook.rs`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_boundary_ingest_records_measured_evidence -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_boundary_mismatch -- --nocapture`.
- Data/schema/provenance: Boundary evidence records transcript path, session id, turn id, tool use id, target paths, and start/end line evidence.
- Runtime: Capture remains local-only and fails closed when Codex does not expose usable transcript evidence.

### GAP-003: There is no canonical unmeasured cost recording path

- Current failure: Unsupported or unavailable usage is described as `unmeasured`, but users have no canonical CLI to append such evidence.
- Good behavior: Given unsupported provider, unavailable transcript, or unknown pricing, when `cost-record unmeasured` runs with target and reason, then it appends an unmeasured receipt with no amount.
- Forbidden behavior: Empty reason, missing target, invalid target, or unmeasured evidence with amount/pricing/usage is accepted.
- Files involved: `rust/task-registry-flow-cli/src/cost_record.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/runtime.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record_unmeasured_appends_reasoned_receipt -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record_unmeasured_rejects_amount_and_missing_reason -- --nocapture`.
- Data/schema/provenance: Unmeasured evidence must include reason and target, and must omit cost amount.
- Runtime: Receipt append is explicit through `--append-receipt`.

### GAP-004: Cost targets are not fully validated

- Current failure: Commit targets are resolved, but plan/task/verifier/release/session targets are accepted as raw strings.
- Good behavior: Given a target kind/id, when cost evidence is recorded, then the target resolves to a known governed object when the kind has local registry authority.
- Forbidden behavior: Unknown plan, task, verifier-run, landing-attempt, release-cycle, empty target id, or invalid commit is accepted.
- Files involved: `rust/task-registry-flow-cli/src/cost_targets.rs`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/cost_record.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets_resolve_governed_objects -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets_reject_unknown_governed_objects -- --nocapture`.
- Data/schema/provenance: Target ids must be canonicalized where possible.
- Runtime: Session targets may remain string ids but must be non-empty and explicit.

### GAP-005: Pricing service tier and reasoning-token semantics are incomplete

- Current failure: Pricing snapshot has one static tier and no explicit reasoning-token billing policy.
- Good behavior: Given standard or fast Codex tier evidence, when ingestion runs, then pricing uses the matching tier and records reasoning-token policy.
- Forbidden behavior: Unknown tier, duplicate model/tier entry, missing reasoning policy when reasoning tokens exist, zero rates, or unpriced model is measured.
- Files involved: `docs/pricing/openai-codex-rate-card-2026-06-02.toml`, `rust/task-registry-flow-cli/src/cost_pricing.rs`, `rust/task-registry-flow-cli/src/cost_ingest.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_calculates_standard_and_fast_tiers -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unknown_tier_and_ambiguous_reasoning -- --nocapture`.
- Data/schema/provenance: Pricing snapshot declares source, version, tier, rates, and reasoning-token policy.
- Runtime: No live pricing lookup; local snapshot only.

### GAP-006: Mutation attribution and cost evidence are not joined

- Current failure: Mutation attribution can identify Codex session/turn/tool, and cost evidence can identify transcript session/turns, but no verifier joins them.
- Good behavior: Given measured Codex mutation attribution, when `cost-coverage-check` runs, then each mutation is covered by matching measured or explicit unmeasured cost evidence.
- Forbidden behavior: Mismatched session, missing turn, wrong target, or absent cost evidence passes as covered.
- Files involved: `rust/task-registry-flow-cli/src/cost_coverage.rs`, `rust/task-registry-flow-cli/src/model_attribution.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_measured_or_unmeasured_mutation_cost -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_uncovered_or_mismatched_mutation -- --nocapture`.
- Data/schema/provenance: Coverage keys include provider/adapter, session id, turn id, target path or governed target.
- Runtime: Coverage check is local and read-only by default.

### GAP-007: Cost report can total invalid measured receipts

- Current failure: `cost-report` parses and totals receipt amounts without first proving replay validity.
- Good behavior: Given valid and invalid measured receipts, when `cost-report --format json` runs, then only replay-valid measured receipts contribute to totals and invalid entries are reported separately.
- Forbidden behavior: Tampered amount, bad pricing hash, overlapping range, or duplicate digest contributes to measured totals.
- Files involved: `rust/task-registry-flow-cli/src/cost_report.rs`, `rust/task-registry-flow-cli/src/cost_evidence.rs`, `rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_totals_only_replay_valid_receipts -- --nocapture`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_excludes_tampered_measured_receipts -- --nocapture`.
- Data/schema/provenance: Report entries include verified, invalid, unmeasured, service tier, and coverage posture.
- Runtime: JSON report returns diagnostic failure when invalid cost evidence exists.

### GAP-008: Product docs can still be overread as automatic token capture

- Current failure: Docs are mostly honest, but do not yet name boundary coverage, service tier, public-safe evidence, and unmeasured record requirements.
- Good behavior: Given public docs, when users read cost posture, then they see governed Codex cost evidence only when usage, pricing, target, boundary, and coverage evidence exist.
- Forbidden behavior: Docs claim automatic token capture, reliable total cost per commit, universal provider attribution, or remote billing truth.
- Files involved: `README.md`, `docs/runtime-schemas.md`, `docs/provider-usage-adapter-contract.md`, `docs/engineering-policy-compliance.md`, `docs/gap-pipeline.md`, `ROADMAP.md`, skill projection files.
- Positive test: `rg -n 'cost-coverage-check|cost-record unmeasured|service tier|boundary|public-safe' README.md docs/runtime-schemas.md docs/provider-usage-adapter-contract.md docs/engineering-policy-compliance.md docs/gap-pipeline.md ROADMAP.md`.
- Negative test: `! rg -n 'automatic token capture|reliable total cost per commit|universal provider attribution|remote billing truth' README.md docs/runtime-schemas.md docs/provider-usage-adapter-contract.md docs/engineering-policy-compliance.md docs/gap-pipeline.md ROADMAP.md`.
- Data/schema/provenance: Docs must preserve measured, estimated, and unmeasured distinctions.
- Runtime: N/A; documentation claim boundary.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage -- --nocapture`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report -- --nocapture`
- `.codex/scripts/task-registry cost-evidence-check --format json`
- `.codex/scripts/task-registry cost-coverage-check --format json`
- `.codex/scripts/task-registry release-check all --format json`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry verify-chain --format json`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml --all-targets -- -D warnings`
- `bash scripts/test-release-readiness.sh all`

## Source File Limit

`rust/task-registry-flow-cli/src/tests/mod.rs` is already at 1600 lines, so no
new test module declarations are allowed in this closure. New tests must fit in
`rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs` or an
approved split must be created before adding a module. Run
`.codex/scripts/task-registry source-limit check` before landing.

## Walkthrough Evidence

- Public release gate rejects private transcript paths.
- Boundary-based measured capture accepts matched Codex boundaries and rejects
  mismatches.
- `cost-record unmeasured` appends reasoned unmeasured evidence and rejects
  amount-bearing unmeasured evidence.
- Pricing tests prove standard and fast tier rates and reasoning-token policy.
- `cost-coverage-check` reports covered and uncovered mutation cost posture.
- `cost-report` excludes invalid measured receipts from totals.
- Final evidence includes source limit, validate, verify-chain, full cargo
  tests, clippy, release readiness, verify-landing, task report, metrics, and
  archive completion.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-06-02-token-capture-hardening"

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-001-public-evidence-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Public cost evidence permits sanitized paths"
given = "Tracked public cost evidence contains only repo-local replayable paths or unmeasured public evidence"
when = "public release cost evidence checks run"
then = "release checks pass without private transcript leakage"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence_allows_sanitized_paths -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence_allows_sanitized_paths -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-002-public-evidence-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Private transcript paths fail release checks"
given = "A tracked measured receipt or public doc includes a private Codex transcript path"
when = "release checks run"
then = "the release check fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence_rejects_private_transcript_paths -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml public_cost_evidence_rejects_private_transcript_paths -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-003-unmeasured-record-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Unmeasured cost receipt records a reason"
given = "A valid governed target has unavailable usage"
when = "cost-record unmeasured runs with a reason and append flag"
then = "an unmeasured receipt is appended without a cost amount"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record_unmeasured_appends_reasoned_receipt -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record_unmeasured_appends_reasoned_receipt -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-004-unmeasured-record-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Invalid unmeasured cost receipt fails"
given = "Unmeasured cost evidence is missing a reason or carries amount evidence"
when = "cost evidence validation runs"
then = "the receipt fails diagnostics"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record_unmeasured_rejects_amount_and_missing_reason -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_record_unmeasured_rejects_amount_and_missing_reason -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-005-target-validation-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Cost targets resolve governed objects"
given = "Registry-backed plan, task, verifier, release, session, and commit targets exist"
when = "cost target resolution runs"
then = "the targets canonicalize successfully"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets_resolve_governed_objects -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets_resolve_governed_objects -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-006-target-validation-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Unknown governed cost targets fail"
given = "A cost receipt targets an unknown governed plan, task, verifier, release, or invalid commit"
when = "target resolution runs"
then = "the runtime rejects the target"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets_reject_unknown_governed_objects -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_targets_reject_unknown_governed_objects -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-007-boundary-capture-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Boundary ingest records matched measured cost"
given = "Codex pre/post boundary evidence and token-count transcript evidence match"
when = "cost ingestion runs from the boundary"
then = "measured cost evidence is recorded with session, turn, tool, and target links"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_boundary_ingest_records_measured_evidence -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_boundary_ingest_records_measured_evidence -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-008-boundary-capture-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Boundary mismatches fail measured capture"
given = "Boundary evidence is missing, reversed, or mismatched by session, turn, tool, target, or range"
when = "cost ingestion attempts measured capture"
then = "the runtime rejects measured cost evidence"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_boundary_mismatch -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_capture_rejects_boundary_mismatch -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-009-tier-pricing-positive"
gap_id = "GAP-005"
polarity = "positive"
title = "Standard and fast tiers calculate separately"
given = "The pricing snapshot contains standard and fast tier rates"
when = "cost pricing runs for each tier"
then = "the calculated amounts differ according to the tier rates"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_calculates_standard_and_fast_tiers -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_calculates_standard_and_fast_tiers -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-010-tier-pricing-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Unknown tier and invalid pricing fail"
given = "A cost request uses an unknown service tier, duplicate tier, or invalid pricing rates"
when = "pricing validation runs"
then = "the runtime fails instead of guessing"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unknown_tier_and_ambiguous_reasoning -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unknown_tier_and_ambiguous_reasoning -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-011-reasoning-positive"
gap_id = "GAP-005"
polarity = "positive"
title = "Reasoning token policy is explicit"
given = "Token usage includes reasoning tokens and the pricing snapshot declares the billing policy"
when = "cost evidence validation replays the receipt"
then = "the receipt passes with explicit reasoning-token semantics"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_calculates_standard_and_fast_tiers -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_calculates_standard_and_fast_tiers -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-012-reasoning-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Ambiguous reasoning token pricing fails"
given = "Token usage includes reasoning tokens without a declared reasoning-token policy"
when = "measured cost evidence is validated"
then = "the receipt fails diagnostics"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unknown_tier_and_ambiguous_reasoning -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_pricing_rejects_unknown_tier_and_ambiguous_reasoning -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-013-coverage-positive"
gap_id = "GAP-006"
polarity = "positive"
title = "Mutation cost coverage accepts measured or unmeasured evidence"
given = "Codex mutation attribution has matching measured or explicit unmeasured cost evidence"
when = "cost-coverage-check runs"
then = "the mutation is reported as covered"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_measured_or_unmeasured_mutation_cost -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_accepts_measured_or_unmeasured_mutation_cost -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-014-coverage-negative"
gap_id = "GAP-006"
polarity = "negative"
title = "Uncovered mutation cost fails coverage"
given = "Codex mutation attribution lacks matching measured or unmeasured cost evidence"
when = "cost-coverage-check runs"
then = "the checker reports a failure"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_uncovered_or_mismatched_mutation -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_coverage_rejects_uncovered_or_mismatched_mutation -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-015-report-positive"
gap_id = "GAP-007"
polarity = "positive"
title = "Cost report totals replay-valid receipts only"
given = "Cost evidence contains replay-valid measured receipts"
when = "cost-report runs"
then = "verified receipts contribute to measured totals"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_totals_only_replay_valid_receipts -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_totals_only_replay_valid_receipts -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-016-report-negative"
gap_id = "GAP-007"
polarity = "negative"
title = "Cost report excludes tampered measured receipts"
given = "Cost evidence contains tampered amount, pricing hash, duplicate digest, or overlapping measured range"
when = "cost-report runs"
then = "invalid receipts are excluded from measured totals and diagnostics fail"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_excludes_tampered_measured_receipts -- --nocapture"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cost_report_excludes_tampered_measured_receipts -- --nocapture"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-017-docs-positive"
gap_id = "GAP-008"
polarity = "positive"
title = "Docs name honest token capture constraints"
given = "Public docs describe token capture"
when = "the docs are inspected"
then = "they name cost coverage, unmeasured records, service tier, boundary evidence, and public-safe evidence"
confirmation = "rg -n 'cost-coverage-check|cost-record unmeasured|service tier|boundary|public-safe' README.md docs/runtime-schemas.md docs/provider-usage-adapter-contract.md docs/engineering-policy-compliance.md docs/gap-pipeline.md ROADMAP.md"

[[behaviors.verifiers]]
type = "command"
command = "rg -n 'cost-coverage-check|cost-record unmeasured|service tier|boundary|public-safe' README.md docs/runtime-schemas.md docs/provider-usage-adapter-contract.md docs/engineering-policy-compliance.md docs/gap-pipeline.md ROADMAP.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-018-docs-negative"
gap_id = "GAP-008"
polarity = "negative"
title = "Docs do not overclaim token capture"
given = "Public docs describe token capture"
when = "forbidden overclaims are searched"
then = "they do not claim automatic token capture, reliable total cost per commit, universal provider attribution, or remote billing truth"
confirmation = "! rg -n 'automatic token capture|reliable total cost per commit|universal provider attribution|remote billing truth' README.md docs/runtime-schemas.md docs/provider-usage-adapter-contract.md docs/engineering-policy-compliance.md docs/gap-pipeline.md ROADMAP.md"

[[behaviors.verifiers]]
type = "command"
command = "! rg -n 'automatic token capture|reliable total cost per commit|universal provider attribution|remote billing truth' README.md docs/runtime-schemas.md docs/provider-usage-adapter-contract.md docs/engineering-policy-compliance.md docs/gap-pipeline.md ROADMAP.md"
expected_exit = 0

[[behaviors]]
behavior_id = "B-TOKEN-CAPTURE-019-token-capture-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Token capture hardening validation passes"
given = "Token capture hardening implementation is present"
when = "focused validation runs"
then = "source limit, cost evidence, coverage, and release checks pass"
confirmation = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry cost-evidence-check --format json && .codex/scripts/task-registry cost-coverage-check --format json && .codex/scripts/task-registry release-check all --format json"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check && .codex/scripts/task-registry cost-evidence-check --format json && .codex/scripts/task-registry cost-coverage-check --format json && .codex/scripts/task-registry release-check all --format json"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-hardening-001"
status = "active"
title = "Make public cost evidence release-safe"
kind = "release"
reason = "Public tracked evidence must not leak private Codex transcript paths or depend on local-only session files."
acceptance_proof = "Behaviors B-TOKEN-CAPTURE-001-public-evidence-positive and B-TOKEN-CAPTURE-002-public-evidence-negative."
behavior_ids = ["B-TOKEN-CAPTURE-001-public-evidence-positive", "B-TOKEN-CAPTURE-002-public-evidence-negative"]

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "public_cost_evidence"
required_change = "Remove private measured transcript path evidence or convert it to public-safe unmeasured evidence."

[[tasks.targets]]
file = "docs/plans/provider-usage-ingestion-2026-06-02.md"
object = "historical_cost_examples"
required_change = "Replace private transcript commands with public-safe canonical examples."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/release_checks.rs"
object = "public_cost_path_gate"
required_change = "Reject private transcript path leakage in tracked public evidence."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "public_cost_path_gate"
required_change = "Fail release readiness on private transcript path leakage."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/release_source_tests.rs"
object = "private_cost_path_tests"
required_change = "Test sanitized public evidence and private transcript rejection."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-hardening-002"
status = "active"
title = "Add target validation and unmeasured recording"
kind = "schema"
reason = "Unknown spend must have a canonical receipt path, and all cost targets must be governed."
acceptance_proof = "Behaviors B-003 through B-006."
behavior_ids = ["B-TOKEN-CAPTURE-003-unmeasured-record-positive", "B-TOKEN-CAPTURE-004-unmeasured-record-negative", "B-TOKEN-CAPTURE-005-target-validation-positive", "B-TOKEN-CAPTURE-006-target-validation-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "cost_evidence_contract"
required_change = "Add cost hardening fields and unmeasured reason support."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_record.rs"
object = "unmeasured_cost_record"
required_change = "Add canonical unmeasured cost recording command."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_targets.rs"
object = "target_resolver"
required_change = "Validate governed cost attribution targets."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "module_list"
required_change = "Include new cost modules."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/runtime.rs"
object = "dispatcher"
required_change = "Route cost-record and cost-coverage-check."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_source"
required_change = "Declare new runtime source files."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-hardening-003"
status = "active"
title = "Add boundary capture and tiered pricing"
kind = "implementation"
reason = "Measured Codex cost must be tied to mutation boundaries and correctly priced by service tier."
acceptance_proof = "Behaviors B-007 through B-012."
behavior_ids = ["B-TOKEN-CAPTURE-007-boundary-capture-positive", "B-TOKEN-CAPTURE-008-boundary-capture-negative", "B-TOKEN-CAPTURE-009-tier-pricing-positive", "B-TOKEN-CAPTURE-010-tier-pricing-negative", "B-TOKEN-CAPTURE-011-reasoning-positive", "B-TOKEN-CAPTURE-012-reasoning-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "codex_cost_boundary"
required_change = "Capture Codex transcript/session/turn/tool boundary evidence."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model_attribution.rs"
object = "boundary_validation"
required_change = "Validate boundary evidence on measured Codex mutation attribution."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_ingest.rs"
object = "boundary_ingest"
required_change = "Support boundary-based ingestion and tier evidence."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_pricing.rs"
object = "pricing_loader"
required_change = "Load tiered pricing and enforce reasoning-token policy."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_evidence.rs"
object = "tiered_replay"
required_change = "Replay measured evidence through tiered pricing semantics."

[[tasks.targets]]
file = "docs/pricing/openai-codex-rate-card-2026-06-02.toml"
object = "tiered_pricing_snapshot"
required_change = "Represent standard and fast Codex tiers."

[[tasks.targets]]
file = ".codex/hooks.json"
object = "codex_hook_projection"
required_change = "Keep pre/post hook projection wired for boundary evidence."

[[tasks.targets]]
file = "hooks/codex-hooks.json"
object = "codex_hook_projection"
required_change = "Mirror hook projection."

[[tasks.targets]]
file = "templates/.codex/hooks.json.template"
object = "codex_hook_projection"
required_change = "Mirror hook template."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-hardening-004"
status = "active"
title = "Add cost coverage and verified reporting"
kind = "validation"
reason = "Cost reports must not total invalid receipts, and mutation cost coverage must be explicit."
acceptance_proof = "Behaviors B-013 through B-016."
behavior_ids = ["B-TOKEN-CAPTURE-013-coverage-positive", "B-TOKEN-CAPTURE-014-coverage-negative", "B-TOKEN-CAPTURE-015-report-positive", "B-TOKEN-CAPTURE-016-report-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_coverage.rs"
object = "coverage_check"
required_change = "Add mutation-to-cost coverage checker."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cost_report.rs"
object = "verified_totals"
required_change = "Total only replay-valid measured receipts and report invalid evidence."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/metrics.rs"
object = "cost_coverage_counts"
required_change = "Count verified, invalid, covered, uncovered, and unmeasured cost posture."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/model.rs"
object = "metrics_report"
required_change = "Add cost coverage count fields."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reviewer_report.rs"
object = "cost_posture_summary"
required_change = "Include honest cost coverage summary."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-hardening-005"
status = "active"
title = "Add token capture hardening tests"
kind = "test"
reason = "The migration requires comprehensive positive and negative behavior coverage."
acceptance_proof = "Behavior B-TOKEN-CAPTURE-019-token-capture-validation."
behavior_ids = ["B-TOKEN-CAPTURE-019-token-capture-validation"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/model_attribution_tests.rs"
object = "token_capture_hardening_tests"
required_change = "Add positive and negative token capture hardening tests."

[[tasks]]
task_id = "TASK-2026-06-02-token-capture-hardening-006"
status = "active"
title = "Document honest token capture posture"
kind = "documentation"
reason = "Public docs and agent skills must reflect measured/unmeasured cost posture without overclaims."
acceptance_proof = "Behaviors B-TOKEN-CAPTURE-017-docs-positive and B-TOKEN-CAPTURE-018-docs-negative."
behavior_ids = ["B-TOKEN-CAPTURE-017-docs-positive", "B-TOKEN-CAPTURE-018-docs-negative"]

[[tasks.targets]]
file = "README.md"
object = "cost_capture_posture"
required_change = "Document governed Codex cost evidence and non-claims."

[[tasks.targets]]
file = "docs/runtime-schemas.md"
object = "cost_capture_schema"
required_change = "Document boundary capture, unmeasured records, tiered pricing, coverage checks, and verified reporting."

[[tasks.targets]]
file = "docs/provider-usage-adapter-contract.md"
object = "adapter_contract"
required_change = "Require service tier, reasoning-token semantics, and public-safe evidence."

[[tasks.targets]]
file = "docs/engineering-policy-compliance.md"
object = "cost_policy_evidence"
required_change = "Document cost evidence as engineering-policy evidence."

[[tasks.targets]]
file = "docs/gap-pipeline.md"
object = "GP-007"
required_change = "Update current token capture evidence and remaining gaps."

[[tasks.targets]]
file = "ROADMAP.md"
object = "token_cost_evidence"
required_change = "Reflect closed hardening and future non-Codex work."

[[tasks.targets]]
file = "skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Add cost-record and cost-coverage guidance."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow.md"
object = "cost_commands"
required_change = "Mirror cost command guidance."

[[tasks.targets]]
file = ".agents/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror cost command guidance."

[[tasks.targets]]
file = ".cursor/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror cost command guidance."

[[tasks.targets]]
file = ".claude/skills/task-registry-flow/SKILL.md"
object = "cost_commands"
required_change = "Mirror cost command guidance."

```
