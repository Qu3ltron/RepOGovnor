# Runtime API Production Hardening Gap Closure Contract

## Approved Scope

Approved production gaps:

- GAP-001: Read-only task-registry commands currently append runtime receipts by default.
- GAP-002: Runtime receipts carry truncated prose instead of typed command, subject, diagnostic, and verifier facts.
- GAP-003: Installer config, manifest policy, action planning, and apply behavior are owned by shell/Python instead of a Rust schema API.
- GAP-004: Mutation hook format is selected by label, while input parsing and output envelopes are not format-typed.
- GAP-005: Status checks are prose shell logic instead of structured runtime diagnostics.
- GAP-006: Completed or cancelled Task Manifest schema version 1 evidence remains accepted by validation.
- GAP-007: Core task-registry commands do not share one JSON command-report API.
- GAP-008: Runtime code is under-capsulized in `main.rs` and `tests.rs`.
- GAP-009: Mutable policy lists remain code-owned where schema/config ownership would make debugging safer.
- GAP-010: Negative migration tests rely too much on prose matching instead of typed JSON fields and side-effect assertions.

Out of scope:

- Hosted UI, remote service integration, analytics export, policy presets, and pull request bot features.
- Compatibility adapters for v1 manifests or v1 receipt lines. Historical evidence may be migrated, but runtime acceptance of legacy schemas is not retained.

Affected surfaces:

- Rust task-registry runtime API, CLI reports, local receipt ledger, mutation hook verification, installer planning/apply flow, status/release checks, docs, templates, and migration tests.

Primitive change gate: N/A. The work restructures existing runtime surfaces and schemas; it does not introduce a new external primitive or network dependency.

## Phased Required Change Checklist

### Phase 0: Activation and safety

- [ ] `[NEW]` `docs/plans/runtime-api-production-hardening.md` - `closure_contract`: add this approved scope and schema version 2 Task Manifest; acceptance proof: `PLAN_ACTIVATE docs/plans/runtime-api-production-hardening.md`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/main.rs` - `thin_dispatch`: reduce `main.rs` to top-level argument parsing, command dispatch, rendering, and process exit; acceptance proof: behavior `B-2026-05-31-G08-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/cli.rs` - `cli_contract`: define global `--format text|json`, `--record-receipt`, command dispatch types, and command result plumbing; acceptance proof: behaviors `B-2026-05-31-G07-positive` and `B-2026-05-31-G07-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/registry.rs` - `registry_api`: move registry load, save, archive, task status, deferral, and metrics domain functions out of `main.rs`; acceptance proof: behavior `B-2026-05-31-G08-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/manifest.rs` - `manifest_api`: move Task Manifest parsing, validation, migration rejection, hash, and plan path handling out of `main.rs`; acceptance proof: behaviors `B-2026-05-31-G06-positive` and `B-2026-05-31-G06-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/verifiers.rs` - `behavior_verifier_api`: move typed verifier execution and verifier results out of `main.rs`; acceptance proof: behavior `B-2026-05-31-G02-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/receipts.rs` - `receipt_api`: own v2 receipt parsing, writing, metrics, and side-effect policy; acceptance proof: behaviors `B-2026-05-31-G01-positive`, `B-2026-05-31-G01-negative`, `B-2026-05-31-G02-positive`, and `B-2026-05-31-G02-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/reports.rs` - `report_rendering`: render text and JSON from typed command reports only; acceptance proof: behavior `B-2026-05-31-G07-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/hook_io.rs` - `hook_format_api`: add format-specific hook input and response schemas; acceptance proof: behaviors `B-2026-05-31-G04-positive` and `B-2026-05-31-G04-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/install.rs` - `installer_runtime_api`: move installer plan/apply schema and action behavior into Rust; acceptance proof: behaviors `B-2026-05-31-G03-positive` and `B-2026-05-31-G03-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/status_checks.rs` - `status_runtime_api`: move install posture and release-source status checks into typed Rust reports; acceptance proof: behaviors `B-2026-05-31-G05-positive` and `B-2026-05-31-G05-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/policy.rs` - `policy_schema_api`: load mutable policy lists from typed config sources where beneficial; acceptance proof: behaviors `B-2026-05-31-G09-positive` and `B-2026-05-31-G09-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/mod.rs` - `test_module_root`: replace monolithic test module entrypoint; acceptance proof: behavior `B-2026-05-31-G08-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/fixtures.rs` - `runtime_test_fixtures`: centralize temporary repo fixtures; acceptance proof: behavior `B-2026-05-31-G10-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/cli_api.rs` - `cli_report_tests`: add JSON command report positives and negatives; acceptance proof: behaviors `B-2026-05-31-G07-positive` and `B-2026-05-31-G07-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/receipts.rs` - `receipt_tests`: add receipt side-effect and v2 schema tests; acceptance proof: behaviors `B-2026-05-31-G01-positive`, `B-2026-05-31-G01-negative`, `B-2026-05-31-G02-positive`, and `B-2026-05-31-G02-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/hook_api.rs` - `hook_api_tests`: add format-typed hook tests; acceptance proof: behaviors `B-2026-05-31-G04-positive` and `B-2026-05-31-G04-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/install_status.rs` - `install_status_tests`: add installer and status JSON tests; acceptance proof: behaviors `B-2026-05-31-G03-positive`, `B-2026-05-31-G03-negative`, `B-2026-05-31-G05-positive`, and `B-2026-05-31-G05-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/migration.rs` - `migration_tests`: add v1 manifest and v1 receipt rejection tests; acceptance proof: behaviors `B-2026-05-31-G06-positive` and `B-2026-05-31-G06-negative`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/release_source.rs` - `release_source_tests`: add release/status typed diagnostic tests; acceptance proof: behavior `B-2026-05-31-G05-positive`.
- [ ] `[NEW]` `rust/task-registry-flow-cli/src/tests/source_limit.rs` - `source_limit_tests`: keep source-limit behavior covered after split; acceptance proof: `.codex/scripts/task-registry source-limit check`.
- [ ] `[DELETE]` `rust/task-registry-flow-cli/src/tests.rs` - `monolithic_tests`: remove after test split; acceptance proof: behavior `B-2026-05-31-G08-negative`.

### Phase 1: Runtime schema and receipt API

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/schema.rs` - `runtime_schema_types`: add command report, command data, runtime surface, diagnostic code, runtime subject, receipt v2, receipt policy, hook decision, install plan report, and status report types; acceptance proof: behavior `B-2026-05-31-G02-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/source_limit.rs` - `typed_source_limit_reports`: return typed diagnostics and command data before text rendering; acceptance proof: behavior `B-2026-05-31-G07-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/release_checks.rs` - `typed_release_reports`: return typed diagnostics and command data before text rendering; acceptance proof: behavior `B-2026-05-31-G05-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/plan_contract.rs` - `policy_backed_contract`: consume plan-contract policy from the policy API where beneficial; acceptance proof: behavior `B-2026-05-31-G09-positive`.
- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/model.rs` - `model_exports`: point registry, manifest, receipt, report, and metrics models at the new schema-backed modules; acceptance proof: behavior `B-2026-05-31-G08-positive`.
- [ ] `[MODIFY]` `docs/task-registry/events.jsonl` - `receipt_ledger_migration`: reset or migrate local receipts to schema version 2 only; acceptance proof: behavior `B-2026-05-31-G02-negative`.

### Phase 2: Hook, installer, and status ownership

- [ ] `[MODIFY]` `rust/task-registry-flow-cli/src/mutation_hook.rs` - `format_typed_authorization`: use `HookFormat` to select concrete input parser and response decision; acceptance proof: behaviors `B-2026-05-31-G04-positive` and `B-2026-05-31-G04-negative`.
- [ ] `[MODIFY]` `tools/agent-governance/pre-tool-use-gap-closure.sh` - `canonical_hook_wrapper`: delegate response envelope generation to Rust; acceptance proof: behavior `B-2026-05-31-G04-positive`.
- [ ] `[MODIFY]` `templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template` - `installed_hook_wrapper`: keep installed projection aligned with canonical hook wrapper; acceptance proof: behavior `B-2026-05-31-G04-positive`.
- [ ] `[MODIFY]` `scripts/render-from-config.sh` - `renderer_wrapper`: become a thin delegating wrapper around Rust installer planning/apply behavior; acceptance proof: behaviors `B-2026-05-31-G03-positive` and `B-2026-05-31-G03-negative`.
- [ ] `[MODIFY]` `scripts/install-to-workspace.sh` - `install_wrapper`: preserve user CLI while delegating runtime policy to Rust; acceptance proof: behavior `B-2026-05-31-G03-positive`.
- [ ] `[MODIFY]` `scripts/status.sh` - `status_wrapper`: delegate posture and release-source checks to Rust status reports; acceptance proof: behavior `B-2026-05-31-G05-positive`.
- [ ] `[MODIFY]` `scripts/release-version-check.sh` - `version_wrapper`: render typed release version diagnostics only; acceptance proof: behavior `B-2026-05-31-G05-positive`.
- [ ] `[MODIFY]` `scripts/release-audit.sh` - `audit_wrapper`: call typed status and release checks from repo root; acceptance proof: behavior `B-2026-05-31-G05-positive`.
- [ ] `[MODIFY]` `scripts/test-install-modes.sh` - `installer_behavior_tests`: assert Rust installer JSON and no dry-run mutation; acceptance proof: behavior `B-2026-05-31-G03-negative`.
- [ ] `[MODIFY]` `scripts/test-release-readiness.sh` - `release_behavior_tests`: assert typed status, release, migration, and wrapper failure behavior; acceptance proof: behaviors `B-2026-05-31-G05-negative` and `B-2026-05-31-G10-negative`.

### Phase 3: Hard-cut migration and docs

- [ ] `[MODIFY]` `docs/plans/production-gap-closures.md` - `task_manifest_v2_migration`: migrate historical manifest to schema version 2 with gap metadata and typed verifiers; acceptance proof: behavior `B-2026-05-31-G06-positive`.
- [ ] `[MODIFY]` `docs/plans/v2-release-readiness.md` - `task_manifest_v2_migration`: migrate historical manifest to schema version 2 with gap metadata and typed verifiers; acceptance proof: behavior `B-2026-05-31-G06-positive`.
- [ ] `[MODIFY]` `docs/plans/v2-production-followup-gap-closures.md` - `task_manifest_v2_migration`: migrate historical manifest to schema version 2 with gap metadata and typed verifiers; acceptance proof: behavior `B-2026-05-31-G06-positive`.
- [ ] `[MODIFY]` `docs/plans/runtime-schema-production-closures.md` - `task_manifest_v2_migration`: migrate historical manifest to schema version 2 with gap metadata and typed verifiers; acceptance proof: behavior `B-2026-05-31-G06-positive`.
- [ ] `[MODIFY]` `docs/task-registry/archive/completed-001.toml` - `archive_hash_refresh`: refresh hashes after historical manifest migration; acceptance proof: `.codex/scripts/task-registry validate`.
- [ ] `[MODIFY]` `docs/runtime-schemas.md` - `runtime_api_reference`: document command reports, receipt v2, hook decisions, installer/status reports, and v2-only manifest policy; acceptance proof: behavior `B-2026-05-31-runtime-api-validation`.
- [ ] `[MODIFY]` `README.md` - `user_runtime_state`: describe read-only commands, receipt recording, and v2-only runtime behavior; acceptance proof: behavior `B-2026-05-31-runtime-api-validation`.
- [ ] `[MODIFY]` `ROADMAP.md` - `roadmap_state`: mark moved schema/status/installer work as current and remove stale future language; acceptance proof: behavior `B-2026-05-31-runtime-api-validation`.
- [ ] `[MODIFY]` `VISION.md` - `clean_runtime_state`: align current state with clean runtime API and no compatibility shims; acceptance proof: behavior `B-2026-05-31-runtime-api-validation`.
- [ ] `[MODIFY]` `CHANGELOG.md` - `release_notes`: record runtime API hardening and v1 runtime cutover; acceptance proof: behavior `B-2026-05-31-runtime-api-validation`.
- [ ] `[MODIFY]` `MANIFEST.toml` - `installer_schema_source`: update only if installer/status command surfaces or tracked files change; acceptance proof: behavior `B-2026-05-31-G09-positive`.
- [ ] `[MODIFY]` `REQUIREMENTS.toml` - `release_schema_source`: update only if release/status tracked surfaces change; acceptance proof: behavior `B-2026-05-31-G09-positive`.

### Phase 4: Verification and handoff

- [ ] `[VERIFY]` `.codex/scripts/task-registry source-limit check` - `source_budget`: prove every source, script, config, doc, template, and governance file stays at or below 1600 lines; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check` - `rust_format`: prove Rust formatting; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings` - `rust_lints`: prove no lint warnings; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml` - `rust_behavior_suite`: prove positive and negative runtime behavior; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `bash scripts/test-install-modes.sh` - `install_migration_suite`: prove installer migration behavior; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `bash scripts/test-release-readiness.sh all` - `release_migration_suite`: prove release/status migration behavior; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 scripts/release-audit.sh` - `release_audit`: prove release audit gate with governed local waiver; acceptance proof: command exits 0.
- [ ] `[VERIFY]` `.codex/scripts/task-registry verify-behaviors PLAN-2026-05-31-runtime-api-production-hardening` - `behavior_contract`: prove all linked behavior verifiers pass; acceptance proof: command exits 0.

## Per-Gap Success Criteria

### GAP-001: Read-only commands mutate runtime state

- Current failure: `main.rs` appends a receipt for every command, so validation and metrics inspection dirty `docs/task-registry/events.jsonl`.
- Good behavior: Given a clean repo when `validate`, `metrics`, `release-check`, `source-limit`, or `verify-behaviors` runs without `--record-receipt`, then `events.jsonl` is unchanged.
- Forbidden behavior: Given the same repo when a read-only command runs without explicit receipt recording, then any appended receipt fails the test.
- Files involved: `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/receipts.rs`, `rust/task-registry-flow-cli/src/main.rs`, `docs/task-registry/events.jsonl`, `rust/task-registry-flow-cli/src/tests/receipts.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_read_only_commands_do_not_mutate_events`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_record_flag_required_for_validation_receipt`.
- Data/schema/provenance: Receipt writes are governed by typed `ReceiptPolicy`, not implicit side effects.
- Runtime: CLI read-only command execution leaves tracked runtime state unchanged unless `--record-receipt` is present.

### GAP-002: Runtime receipts hide structured facts in prose

- Current failure: `ReceiptEvent.detail` is a truncated string with no typed subject, diagnostic, verifier, or denial shape.
- Good behavior: Given a mutating command, verifier run, or denied mutation, when a receipt is recorded, then schema version 2 includes typed command, subject, outcome, diagnostics, verifier results, and mutation denial data as applicable.
- Forbidden behavior: Given a schema version 1 receipt or a detail-only receipt, metrics and validation reject or count it as malformed according to typed rules.
- Files involved: `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/receipts.rs`, `rust/task-registry-flow-cli/src/verifiers.rs`, `rust/task-registry-flow-cli/src/tests/receipts.rs`, `docs/runtime-schemas.md`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_v2_serializes_typed_subjects`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_rejects_schema_v1_receipts`.
- Data/schema/provenance: `ReceiptEventV2` is `deny_unknown_fields`; local receipts remain local repo files.
- Runtime: Metrics distinguish failed events, mutation denials, malformed receipts, and schema-version failures without parsing prose.

### GAP-003: Installer is not schema-owned

- Current failure: `render-from-config.sh` embeds project config schema, manifest path validation, action vocabulary, stale paths, dry-run JSON, and apply behavior in Python.
- Good behavior: Given a valid config and manifest, when `task-registry-flow install plan|apply` runs, then Rust emits or applies typed install actions that match `MANIFEST.toml` policy.
- Forbidden behavior: Given unknown config fields, noncanonical runtime paths, unknown manifest fields, unknown action values, legacy skill symlinks, or dry-run mode, the installer fails closed or preserves no mutation as specified.
- Files involved: `rust/task-registry-flow-cli/src/install.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `scripts/render-from-config.sh`, `scripts/install-to-workspace.sh`, `MANIFEST.toml`, `rust/task-registry-flow-cli/src/tests/install_status.rs`, `scripts/test-install-modes.sh`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_plan_emits_typed_actions`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_rejects_unknown_config_and_dry_run_mutation`.
- Data/schema/provenance: Installer action vocabulary is loaded from typed manifest policy and verified against Rust enums.
- Runtime: `install-to-workspace.sh --dry-run|--merge|--force` remains available but delegates runtime policy to Rust.

### GAP-004: Hook API is not format-typed

- Current failure: `verify_mutation_hook` ignores the supplied `HookFormat`, and shell/Python builds hook response envelopes.
- Good behavior: Given Codex, Cursor, or Antigravity payloads, when the matching `HookFormat` is used, then the Rust runtime parses the concrete payload and emits the concrete response envelope.
- Forbidden behavior: Given a payload for the wrong format, ambiguous write-intent payload, outside-root path, or arbitrary recursive key that looks like a path, the hook fails closed.
- Files involved: `rust/task-registry-flow-cli/src/hook_io.rs`, `rust/task-registry-flow-cli/src/mutation_hook.rs`, `tools/agent-governance/pre-tool-use-gap-closure.sh`, `templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template`, `rust/task-registry-flow-cli/src/tests/hook_api.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_format_typed_payloads_emit_responses`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_rejects_format_mismatch_and_uncertain_paths`.
- Data/schema/provenance: `HookDecision` and hook response structs are typed Rust schema values.
- Runtime: The hook script contains no Python JSON response builder.

### GAP-005: Status remains prose shell runtime

- Current failure: `status.sh` owns counters, grep checks, embedded Python reads, and release-source behavior.
- Good behavior: Given a workspace, when `status-check --format json` runs, then install posture, release-source, tracked-for-CI, marker, hook, skill, and stale-path facts are typed diagnostics.
- Forbidden behavior: Given missing native skills, stale paths, malformed markers, or untracked required files, status fails with typed check ids and does not rely on grep-only prose.
- Files involved: `rust/task-registry-flow-cli/src/status_checks.rs`, `rust/task-registry-flow-cli/src/release_checks.rs`, `rust/task-registry-flow-cli/src/reports.rs`, `scripts/status.sh`, `scripts/release-version-check.sh`, `scripts/release-audit.sh`, `rust/task-registry-flow-cli/src/tests/install_status.rs`, `rust/task-registry-flow-cli/src/tests/release_source.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_reports_marker_skill_hook_ci_facts`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_fails_missing_native_skill_projection`.
- Data/schema/provenance: Status surfaces emit `CheckReport` or `StatusReport` with typed `DiagnosticCode` values.
- Runtime: Shell status remains a user wrapper over Rust status facts.

### GAP-006: Legacy v1 manifests still pass completed or cancelled state

- Current failure: validation accepts schema version 1 Task Manifests when related registry plans are completed or cancelled.
- Good behavior: Given any Task Manifest, when validation parses it, then only schema version 2 is accepted.
- Forbidden behavior: Given completed or cancelled historical v1 manifests, validation still fails until migrated.
- Files involved: `rust/task-registry-flow-cli/src/manifest.rs`, `rust/task-registry-flow-cli/src/main.rs`, `docs/plans/production-gap-closures.md`, `docs/plans/v2-release-readiness.md`, `docs/plans/v2-production-followup-gap-closures.md`, `docs/plans/runtime-schema-production-closures.md`, `docs/task-registry/archive/completed-001.toml`, `rust/task-registry-flow-cli/src/tests/migration.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml validation_accepts_migrated_v2_historical_manifests`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml validation_rejects_completed_legacy_v1_manifest`.
- Data/schema/provenance: Plan hashes in the archive are refreshed through registry validation after manifest migration.
- Runtime: `.codex/scripts/task-registry validate` has no v1 compatibility path.

### GAP-007: Core CLI has no clean JSON API for most commands

- Current failure: most command success and failure states are available only through human text.
- Good behavior: Given any command, when `--format json` is used, then one `CommandReport` envelope is emitted with typed command, status, data, diagnostics, and receipt policy.
- Forbidden behavior: Given unknown format, trailing args, or invalid subcommand shape, the CLI emits typed failure and exits nonzero.
- Files involved: `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/reports.rs`, `rust/task-registry-flow-cli/src/schema.rs`, `rust/task-registry-flow-cli/src/tests/cli_api.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_envelope_all_commands`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_rejects_unknown_format_and_trailing_args`.
- Data/schema/provenance: JSON reports are schema versioned and `deny_unknown_fields` where parsed.
- Runtime: Text output is a renderer, not the command contract.

### GAP-008: Main runtime is under-capsulized

- Current failure: `main.rs` mixes dispatch, registry persistence, validation, manifest parsing, verifiers, metrics, receipts, and path logic; `tests.rs` is monolithic.
- Good behavior: Given the runtime source, when files are inspected, then domain logic is in named modules and `main.rs` is a thin dispatcher under 300 lines.
- Forbidden behavior: New installer, status, hook, registry, or manifest policy remains embedded in `main.rs` or a monolithic `tests.rs`.
- Files involved: `rust/task-registry-flow-cli/src/main.rs`, `rust/task-registry-flow-cli/src/registry.rs`, `rust/task-registry-flow-cli/src/manifest.rs`, `rust/task-registry-flow-cli/src/verifiers.rs`, `rust/task-registry-flow-cli/src/cli.rs`, `rust/task-registry-flow-cli/src/tests/mod.rs`, `rust/task-registry-flow-cli/src/tests/*.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml module_split_preserves_registry_behaviors`.
- Negative test: `.codex/scripts/task-registry source-limit check`.
- Data/schema/provenance: Module boundaries are source-level runtime API boundaries, not docs-only guidance.
- Runtime: Existing CLI behaviors remain available through module-owned APIs.

### GAP-009: Policy constants are code-owned

- Current failure: source-limit coverage, plan sections/placeholders, install stale paths/actions, and release/tracked lists are partly duplicated across code and scripts.
- Good behavior: Given policy-bearing config files, when runtime checks run, then mutable policy lists are loaded from typed config sources where beneficial.
- Forbidden behavior: Unknown policy fields, action-vocabulary mismatches, or duplicated stale-path policy can pass silently.
- Files involved: `rust/task-registry-flow-cli/src/policy.rs`, `rust/task-registry-flow-cli/src/source_limit.rs`, `rust/task-registry-flow-cli/src/plan_contract.rs`, `rust/task-registry-flow-cli/src/install.rs`, `MANIFEST.toml`, `REQUIREMENTS.toml`, `rust/task-registry-flow-cli/src/tests/install_status.rs`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml policy_loads_contract_sections_and_action_vocabulary`.
- Negative test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml policy_rejects_unknown_fields_and_action_mismatch`.
- Data/schema/provenance: Policy files are parsed with typed schemas and unknown-field rejection.
- Runtime: Adding policy requires config/schema changes, not loose prose edits in scripts.

### GAP-010: Negative tests assert prose too often

- Current failure: migration and wrapper tests frequently grep human output instead of asserting typed fields and file-state boundaries.
- Good behavior: Given negative cases, tests parse JSON fields or inspect file-system side effects with exact expectations.
- Forbidden behavior: A migration negative test passes solely because a human sentence matched.
- Files involved: `rust/task-registry-flow-cli/src/tests/*.rs`, `scripts/test-install-modes.sh`, `scripts/test-release-readiness.sh`.
- Positive test: `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml negative_tests_parse_json_contracts`.
- Negative test: `bash scripts/test-release-readiness.sh all`.
- Data/schema/provenance: Negative suites assert check ids, diagnostic status, subjects, action enums, and receipt policy.
- Runtime: Shell greps remain allowed only for wrapper usage smoke checks.

## Validation Plan

Focused:

- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml migration_`
- `.codex/scripts/task-registry source-limit check`

Full:

- `.codex/scripts/task-registry validate`
- `.codex/scripts/task-registry metrics --format json`
- `cargo fmt --manifest-path rust/task-registry-flow-cli/Cargo.toml -- --check`
- `cargo clippy --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml -- -D warnings`
- `cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml`
- `bash scripts/test-install-modes.sh`
- `bash scripts/test-release-readiness.sh all`
- `AGENT_GOVERNANCE_ALLOW_AUDIT_TOOL_WAIVER=1 scripts/release-audit.sh`

Failure evidence:

- Any schema version 1 Task Manifest remains in `docs/plans`.
- Any schema version 1 receipt remains in `docs/task-registry/events.jsonl`.
- Any read-only command mutates receipts without `--record-receipt`.
- Any hook wrapper builds runtime JSON response envelopes outside Rust.
- Any installer/status policy remains authoritative in shell/Python instead of Rust typed reports.
- Any negative migration test relies only on prose output instead of JSON fields or file-state checks.

## Walkthrough Evidence

Capture:

- `PLAN_ACTIVATE docs/plans/runtime-api-production-hardening.md` output.
- Focused positive and negative behavior verifier outputs.
- Full validation command outputs from the Validation Plan.
- `.codex/scripts/task-registry report PLAN-2026-05-31-runtime-api-production-hardening`.
- `.codex/scripts/task-registry metrics --format json`.
- Final source-limit evidence showing every touched file is at or below 1600 lines.

## Task Manifest

```toml
schema_version = 2
plan_id = "PLAN-2026-05-31-runtime-api-production-hardening"

[[behaviors]]
behavior_id = "B-2026-05-31-G01-positive"
gap_id = "GAP-001"
polarity = "positive"
title = "Read-only commands are side-effect free by default"
given = "A repository with an existing task registry receipt ledger"
when = "Read-only commands run without the record receipt flag"
then = "The receipt ledger is unchanged"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_read_only_commands_do_not_mutate_events"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_read_only_commands_do_not_mutate_events"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G01-negative"
gap_id = "GAP-001"
polarity = "negative"
title = "Receipt recording requires explicit opt in for read-only commands"
given = "A read-only validation command"
when = "The command runs without explicit receipt recording"
then = "No validation receipt is appended"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_record_flag_required_for_validation_receipt"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_record_flag_required_for_validation_receipt"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G02-positive"
gap_id = "GAP-002"
polarity = "positive"
title = "Receipts serialize typed v2 subjects"
given = "A mutating command or verifier result is recorded"
when = "The receipt is serialized"
then = "The receipt includes typed command, subject, outcome, diagnostics, and verifier data"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_v2_serializes_typed_subjects"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml receipt_v2_serializes_typed_subjects"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G02-negative"
gap_id = "GAP-002"
polarity = "negative"
title = "Schema version 1 receipts are rejected"
given = "A local receipt ledger containing a schema version 1 line"
when = "Metrics read the ledger"
then = "The line is counted or rejected as malformed schema evidence"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_rejects_schema_v1_receipts"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml metrics_rejects_schema_v1_receipts"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G03-positive"
gap_id = "GAP-003"
polarity = "positive"
title = "Installer plan emits typed Rust actions"
given = "A valid project config and plugin manifest"
when = "The Rust installer plans an install"
then = "The result is a typed install action report"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_plan_emits_typed_actions"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_plan_emits_typed_actions"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G03-negative"
gap_id = "GAP-003"
polarity = "negative"
title = "Installer rejects unknown config and dry-run mutation"
given = "An invalid config or dry-run installer invocation"
when = "The installer evaluates the request"
then = "Unknown config fails closed and dry-run leaves files unchanged"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_rejects_unknown_config_and_dry_run_mutation"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml installer_rejects_unknown_config_and_dry_run_mutation"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G04-positive"
gap_id = "GAP-004"
polarity = "positive"
title = "Hook formats parse and respond through typed Rust schemas"
given = "Codex, Cursor, and Antigravity hook payloads"
when = "The matching hook format is selected"
then = "The Rust runtime parses targets and emits the matching response envelope"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_format_typed_payloads_emit_responses"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_format_typed_payloads_emit_responses"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G04-negative"
gap_id = "GAP-004"
polarity = "negative"
title = "Hook rejects format mismatch and uncertain paths"
given = "A mismatched hook payload or ambiguous write request"
when = "The hook verifier runs"
then = "The request fails closed with typed denial data"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_rejects_format_mismatch_and_uncertain_paths"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml hook_rejects_format_mismatch_and_uncertain_paths"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G05-positive"
gap_id = "GAP-005"
polarity = "positive"
title = "Status checks emit typed diagnostics"
given = "A governed workspace"
when = "Status checks run with JSON output"
then = "Marker, hook, skill, tracked-for-CI, and release-source facts are typed diagnostics"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_reports_marker_skill_hook_ci_facts"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_reports_marker_skill_hook_ci_facts"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G05-negative"
gap_id = "GAP-005"
polarity = "negative"
title = "Status checks fail missing native skill projections"
given = "A workspace with legacy symlinked skill projections"
when = "Status checks run"
then = "The check fails with a typed native-skill diagnostic"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_fails_missing_native_skill_projection"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml status_check_fails_missing_native_skill_projection"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G06-positive"
gap_id = "GAP-006"
polarity = "positive"
title = "Migrated historical manifests validate as v2"
given = "Historical task plans have migrated Task Manifests"
when = "Registry validation runs"
then = "All historical manifests validate as schema version 2"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml validation_accepts_migrated_v2_historical_manifests"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml validation_accepts_migrated_v2_historical_manifests"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G06-negative"
gap_id = "GAP-006"
polarity = "negative"
title = "Completed legacy v1 manifests are rejected"
given = "A completed registry plan still has a schema version 1 Task Manifest"
when = "Validation parses the manifest"
then = "Validation fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml validation_rejects_completed_legacy_v1_manifest"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml validation_rejects_completed_legacy_v1_manifest"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G07-positive"
gap_id = "GAP-007"
polarity = "positive"
title = "All commands emit JSON command envelopes"
given = "Core task-registry commands"
when = "They run with JSON output"
then = "Each emits one typed command report"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_envelope_all_commands"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_json_envelope_all_commands"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G07-negative"
gap_id = "GAP-007"
polarity = "negative"
title = "Invalid CLI format and trailing args fail closed"
given = "A command with an unknown output format or unexpected trailing args"
when = "The CLI parses the command"
then = "The command emits a typed failure and exits nonzero"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_rejects_unknown_format_and_trailing_args"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml cli_rejects_unknown_format_and_trailing_args"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G08-positive"
gap_id = "GAP-008"
polarity = "positive"
title = "Module split preserves runtime behavior"
given = "The Rust runtime has domain modules"
when = "Registry behavior tests run"
then = "Existing registry behaviors pass through module-owned APIs"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml module_split_preserves_registry_behaviors"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml module_split_preserves_registry_behaviors"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G08-negative"
gap_id = "GAP-008"
polarity = "negative"
title = "Monolithic runtime files stay below source budget"
given = "Runtime source files after the split"
when = "The source-limit gate runs"
then = "No source file exceeds the hard 1600-line budget"
confirmation = ".codex/scripts/task-registry source-limit check"

[[behaviors.verifiers]]
type = "command"
command = ".codex/scripts/task-registry source-limit check"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G09-positive"
gap_id = "GAP-009"
polarity = "positive"
title = "Policy loads from typed config sources"
given = "Plan contract, source-limit, installer, and release policy files"
when = "Policy loaders parse them"
then = "The runtime receives typed policy values"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml policy_loads_contract_sections_and_action_vocabulary"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml policy_loads_contract_sections_and_action_vocabulary"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G09-negative"
gap_id = "GAP-009"
polarity = "negative"
title = "Policy rejects unknown fields and action mismatches"
given = "A policy file with unknown fields or mismatched action vocabulary"
when = "The runtime parses policy"
then = "Parsing fails closed"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml policy_rejects_unknown_fields_and_action_mismatch"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml policy_rejects_unknown_fields_and_action_mismatch"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G10-positive"
gap_id = "GAP-010"
polarity = "positive"
title = "Negative tests parse JSON contracts"
given = "Negative migration and wrapper fixtures"
when = "The negative suites run"
then = "They assert typed JSON fields or exact file-state boundaries"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml negative_tests_parse_json_contracts"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml negative_tests_parse_json_contracts"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-G10-negative"
gap_id = "GAP-010"
polarity = "negative"
title = "Release readiness rejects prose-only migration assertions"
given = "Release readiness negative fixtures"
when = "The migration suite runs"
then = "Failures are proven by JSON fields or state assertions, not prose-only matching"
confirmation = "bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[behaviors]]
behavior_id = "B-2026-05-31-runtime-api-validation"
gap_id = "GAP-VALIDATION"
polarity = "validation"
title = "Full runtime API validation passes"
given = "The implementation is complete"
when = "The full validation gate runs"
then = "Registry, tests, install, release, audit, and source-limit gates pass"
confirmation = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-install-modes.sh && bash scripts/test-release-readiness.sh all"

[[behaviors.verifiers]]
type = "command"
command = "cargo test --locked --manifest-path rust/task-registry-flow-cli/Cargo.toml && bash scripts/test-install-modes.sh && bash scripts/test-release-readiness.sh all"
expected_exit = 0

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-001"
title = "Make receipt writes explicit and side-effect safe"
status = "planned"
kind = "schema"
reason = "Read-only runtime commands append local receipts without explicit authorization."
acceptance_proof = "Behaviors B-2026-05-31-G01-positive and B-2026-05-31-G01-negative pass."
behavior_ids = ["B-2026-05-31-G01-positive", "B-2026-05-31-G01-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "receipt_policy_flags"
required_change = "Parse and pass record-receipt policy explicitly."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/receipts.rs"
object = "read_only_receipt_policy"
required_change = "Write receipts only for mutating commands by default and only for read-only commands when requested."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "receipt_dispatch"
required_change = "Remove unconditional append_event behavior from command exit handling."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/receipts.rs"
object = "receipt_side_effect_tests"
required_change = "Assert read-only commands do not mutate events without record-receipt."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-002"
title = "Replace prose receipts with typed v2 receipt schema"
status = "planned"
kind = "schema"
reason = "Receipt diagnostics and verifier facts are hidden in truncated prose."
acceptance_proof = "Behaviors B-2026-05-31-G02-positive and B-2026-05-31-G02-negative pass."
behavior_ids = ["B-2026-05-31-G02-positive", "B-2026-05-31-G02-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "receipt_v2_schema"
required_change = "Define typed receipt v2 subjects, outcome data, diagnostics, verifier results, and mutation denial data."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/receipts.rs"
object = "receipt_v2_parser_writer"
required_change = "Serialize v2 receipts and reject or count legacy receipt lines as malformed."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/verifiers.rs"
object = "typed_verifier_results"
required_change = "Return verifier result data that can be embedded in reports and receipts."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/receipts.rs"
object = "receipt_schema_tests"
required_change = "Assert typed v2 receipt structure and legacy receipt rejection."

[[tasks.targets]]
file = "docs/task-registry/events.jsonl"
object = "receipt_ledger_migration"
required_change = "Reset or migrate local receipt lines to schema version 2 only."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-003"
title = "Move installer behavior into Rust schema API"
status = "planned"
kind = "migration"
reason = "Installer behavior is duplicated in shell/Python instead of one clean runtime API."
acceptance_proof = "Behaviors B-2026-05-31-G03-positive and B-2026-05-31-G03-negative pass."
behavior_ids = ["B-2026-05-31-G03-positive", "B-2026-05-31-G03-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/install.rs"
object = "installer_runtime_api"
required_change = "Parse config and manifest, produce typed install plans, and apply actions from Rust."

[[tasks.targets]]
file = "scripts/render-from-config.sh"
object = "installer_renderer_wrapper"
required_change = "Delegate renderer work to Rust installer commands."

[[tasks.targets]]
file = "scripts/install-to-workspace.sh"
object = "installer_user_wrapper"
required_change = "Preserve install CLI while delegating runtime policy to Rust."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "installer_migration_suite"
required_change = "Assert typed installer JSON, dry-run immutability, and symlink replacement."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/install_status.rs"
object = "installer_unit_tests"
required_change = "Add positive and negative Rust installer behavior tests."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-004"
title = "Make mutation hook input and output format typed"
status = "planned"
kind = "authorization"
reason = "Hook format is currently not a real parser or response contract."
acceptance_proof = "Behaviors B-2026-05-31-G04-positive and B-2026-05-31-G04-negative pass."
behavior_ids = ["B-2026-05-31-G04-positive", "B-2026-05-31-G04-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/hook_io.rs"
object = "hook_format_schemas"
required_change = "Define typed input parsers and output envelopes for Codex, Cursor, and Antigravity."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/mutation_hook.rs"
object = "format_typed_authorization"
required_change = "Use HookFormat to parse and authorize concrete hook payloads."

[[tasks.targets]]
file = "tools/agent-governance/pre-tool-use-gap-closure.sh"
object = "canonical_hook_wrapper"
required_change = "Delegate hook response JSON to Rust."

[[tasks.targets]]
file = "templates/tools/agent-governance/pre-tool-use-gap-closure.sh.template"
object = "installed_hook_wrapper"
required_change = "Align installed hook wrapper with canonical Rust response generation."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/hook_api.rs"
object = "hook_format_tests"
required_change = "Add positive and negative hook format tests."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-005"
title = "Move status checks to typed Rust diagnostics"
status = "planned"
kind = "diagnostics"
reason = "Status and release checks are currently shell-owned prose checks."
acceptance_proof = "Behaviors B-2026-05-31-G05-positive and B-2026-05-31-G05-negative pass."
behavior_ids = ["B-2026-05-31-G05-positive", "B-2026-05-31-G05-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/status_checks.rs"
object = "status_check_reports"
required_change = "Emit typed install, environment, tracked-for-CI, stale-path, and release-source diagnostics."

[[tasks.targets]]
file = "scripts/status.sh"
object = "status_wrapper"
required_change = "Delegate status logic to Rust and render the report."

[[tasks.targets]]
file = "scripts/release-version-check.sh"
object = "version_wrapper"
required_change = "Delegate version consistency to typed Rust reports."

[[tasks.targets]]
file = "scripts/release-audit.sh"
object = "audit_wrapper"
required_change = "Use Rust status and release checks from repository root."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/release_source.rs"
object = "release_status_tests"
required_change = "Assert release and status diagnostics by typed check id."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-006"
title = "Hard cut all Task Manifests to schema version 2"
status = "planned"
kind = "migration"
reason = "Runtime validation still accepts completed or cancelled schema version 1 manifest evidence."
acceptance_proof = "Behaviors B-2026-05-31-G06-positive and B-2026-05-31-G06-negative pass."
behavior_ids = ["B-2026-05-31-G06-positive", "B-2026-05-31-G06-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/manifest.rs"
object = "v2_only_manifest_validation"
required_change = "Reject schema version 1 manifests in every registry status."

[[tasks.targets]]
file = "docs/plans/production-gap-closures.md"
object = "historical_manifest_v2"
required_change = "Migrate Task Manifest to schema version 2 with gap metadata and typed verifiers."

[[tasks.targets]]
file = "docs/plans/v2-release-readiness.md"
object = "historical_manifest_v2"
required_change = "Migrate Task Manifest to schema version 2 with gap metadata and typed verifiers."

[[tasks.targets]]
file = "docs/plans/v2-production-followup-gap-closures.md"
object = "historical_manifest_v2"
required_change = "Migrate Task Manifest to schema version 2 with gap metadata and typed verifiers."

[[tasks.targets]]
file = "docs/plans/runtime-schema-production-closures.md"
object = "historical_manifest_v2"
required_change = "Migrate Task Manifest to schema version 2 with gap metadata and typed verifiers."

[[tasks.targets]]
file = "docs/task-registry/archive/completed-001.toml"
object = "migrated_plan_hashes"
required_change = "Refresh archived plan hashes after manifest migration."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/migration.rs"
object = "v1_rejection_tests"
required_change = "Assert completed schema version 1 manifests fail validation."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-007"
title = "Add JSON command report API to all commands"
status = "planned"
kind = "schema"
reason = "Core command results are still mostly text-only."
acceptance_proof = "Behaviors B-2026-05-31-G07-positive and B-2026-05-31-G07-negative pass."
behavior_ids = ["B-2026-05-31-G07-positive", "B-2026-05-31-G07-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/cli.rs"
object = "json_command_envelope"
required_change = "Parse global format flags and return one command report envelope for every command."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/reports.rs"
object = "command_report_renderer"
required_change = "Render text and JSON from typed command reports."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/schema.rs"
object = "command_report_schema"
required_change = "Define command report and command data schemas."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/cli_api.rs"
object = "json_cli_tests"
required_change = "Assert JSON envelope success and parser failure behavior."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-008"
title = "Capsulize Rust runtime modules"
status = "planned"
kind = "implementation"
reason = "Runtime domains are concentrated in monolithic source and test files."
acceptance_proof = "Behaviors B-2026-05-31-G08-positive and B-2026-05-31-G08-negative pass."
behavior_ids = ["B-2026-05-31-G08-positive", "B-2026-05-31-G08-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/registry.rs"
object = "registry_domain"
required_change = "Own registry lifecycle, task status, archive, metrics, and persistence behavior."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/manifest.rs"
object = "manifest_domain"
required_change = "Own manifest parsing, validation, hash, and plan path behavior."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/main.rs"
object = "thin_runtime_entrypoint"
required_change = "Keep only module declarations, top-level dispatch wiring, and process exit."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/mod.rs"
object = "split_test_root"
required_change = "Load focused test modules instead of a monolithic tests file."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests.rs"
object = "remove_monolith"
required_change = "Delete after tests are moved into focused modules."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-009"
title = "Load mutable policy from typed config"
status = "planned"
kind = "schema"
reason = "Mutable policy lists are duplicated across code, scripts, and config."
acceptance_proof = "Behaviors B-2026-05-31-G09-positive and B-2026-05-31-G09-negative pass."
behavior_ids = ["B-2026-05-31-G09-positive", "B-2026-05-31-G09-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/policy.rs"
object = "typed_policy_loaders"
required_change = "Load plan contract, source-limit, installer, release, and tracked-for-CI policy from typed config sources."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/plan_contract.rs"
object = "contract_policy_loader"
required_change = "Use policy API for required sections and placeholder rules where beneficial."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/source_limit.rs"
object = "source_policy_loader"
required_change = "Use policy API for source-limit coverage and exclusions where beneficial."

[[tasks.targets]]
file = "MANIFEST.toml"
object = "installer_policy_source"
required_change = "Remain the typed source for installer action vocabulary and stale install paths."

[[tasks.targets]]
file = "REQUIREMENTS.toml"
object = "release_policy_source"
required_change = "Remain the typed source for release and tracked-for-CI policy."

[[tasks]]
task_id = "TASK-2026-05-31-runtime-api-010"
title = "Convert negative migration tests to typed assertions"
status = "planned"
kind = "test"
reason = "Several migration tests prove behavior through prose grep instead of typed contracts."
acceptance_proof = "Behaviors B-2026-05-31-G10-positive and B-2026-05-31-G10-negative pass."
behavior_ids = ["B-2026-05-31-G10-positive", "B-2026-05-31-G10-negative"]

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/fixtures.rs"
object = "typed_negative_fixtures"
required_change = "Provide fixtures that assert JSON fields and file-state boundaries."

[[tasks.targets]]
file = "rust/task-registry-flow-cli/src/tests/cli_api.rs"
object = "typed_negative_cli_tests"
required_change = "Assert parser and command failures through typed JSON reports."

[[tasks.targets]]
file = "scripts/test-release-readiness.sh"
object = "typed_negative_release_tests"
required_change = "Parse JSON diagnostics or inspect state for migration negatives."

[[tasks.targets]]
file = "scripts/test-install-modes.sh"
object = "typed_negative_install_tests"
required_change = "Parse installer JSON for action and dry-run mutation assertions."
```
